from itertools import count
from collections import Counter
import random
from scipy.interpolate import interp1d
from scipy import interpolate
import numpy as np
import matplotlib.pyplot as plt
from matplotlib import style
from math import exp
from functools import wraps

style.use("seaborn")


def interpolate_spline(xs, ys, s=0, der=0):
    tck = interpolate.splrep(xs, ys, s=s)
    return lambda xs: interpolate.splev(xs, tck, der=der)


# source: https://www.statista.com/statistics/454349/population-by-age-group-germany/
population_age_distribution = {  # in millions
    0: 0,
    1: 0.78,
    5: 3.88,
    14: 6.22,
    17: 2.31,
    20: 2.59,
    24: 3.72,
    39: 15.84,
    59: 23.9,
    64: 5.49,
    100: 17.88,
}

number_of_people = sum(population_age_distribution.values())
# m = max(population_age_distribution.values())
age_percentages = {}
s = 0
for key, value in population_age_distribution.items():
    s += value / number_of_people
    age_percentages[key] = s

print(age_percentages)
xs = list(age_percentages.keys())  # ages
ys = list(age_percentages.values())  # folded probabilities
# function that can be samples with random values from 0 to 1 to produce an age between 0 and 100 corresponding to the age distribution of germany's population
get_age = interp1d(ys, xs, kind="cubic")
# get_age = interpolate_spline(ys, xs) # alternative version using spline interpolation
""" just a small test of the function above
xs = np.linspace(0, 1, 10000)
plt.plot(xs, get_age(xs))
plt.show()
v = 0
sample = 100000
for _ in range(sample):
    if 25 < get_age(random.random()) <= 39:
        v += 1

print(v / sample)
"""


def clamp(min_, max_, x):
    return max(min_, min(max_, x))


def mult(factor):
    def decorator(f):
        @wraps(f)
        def decorated(*args, **kwargs):
            return factor * f(*args, **kwargs)
        return decorated
    return decorator


class Person():

    def __init__(self):
        self.age = get_age(random.random())
        self.days_since_infection = 0

    def cure_function(self):
        return 1/(self.age/10) * (1 - exp(-self.days_since_infection / 10))

    @staticmethod
    def number_of_people_met(day):
        if day <= 7:
            return 4
        elif day <= 21:
            return 2
        else:
            return 1

    @mult(0.065)
    def death_chance(self):
        # source: worldometers.info COVID-19 Fatality rate by age
        age = self.age
        if age <= 9:
            return 0
        elif age <= 39:
            return 0.002
        elif age <= 49:
            return 0.004
        elif age <= 59:
            return 0.013
        elif age <= 69:
            return 0.036
        elif age <= 79:
            return 0.08
        else:
            return 0.148

    @mult(0.28)
    def infection_chance(self):
        # just guessed some values
        age = self.age
        if age <= 9:
            return 0.05
        elif age <= 19:
            return 0.1
        elif age <= 39:
            return 0.2
        else:
            return 0.4

    def cure_chance(self):
        return clamp(0, 1, self.cure_function())

    def gets_cured(self):
        y = random.random() < self.cure_chance()
        return y

    def dies(self):
        return random.random() < self.death_chance()


def age_distribution(population):
    return Counter(map(lambda p: 10 * round(p.age / 10), population))


deaths = []
death_tolls = [0]
while len(deaths) == 0:
    infections = []
    populations = [1]
    population = [Person()]
    cured = []
    for day in count(0):
        new_population = []
        for person in population:
            person.days_since_infection += 1
            if person.dies():
                deaths.append(person)
                continue
            elif person.gets_cured():
                cured.append(person)
                continue
            else:
                new_population.append(person)
                # infect other people
                for _ in range(Person.number_of_people_met(person.days_since_infection)):
                    p = Person()
                    if random.random() < p.infection_chance():
                        new_population.append(p)
                        infections.append(p)
        population = new_population
        populations.append(len(new_population))
        death_tolls.append(len(deaths))
        if day >= 75 or len(population) > 200000:
            break


plt.subplot(2, 1, 1)
plt.semilogy(range(len(populations)), populations, label="Currently Infected")
plt.semilogy(range(len(populations)), death_tolls, label="Dead")
plt.xlabel("Time in days")
plt.ylabel("Number of People")
plt.legend()
plt.subplot(2, 3, 4)
ages = age_distribution(infections)
plt.bar(ages.keys(), ages.values())
plt.xlabel("Age in years")
plt.ylabel("Number of Infections")
plt.subplot(2, 3, 5)
ages = age_distribution(deaths)
plt.bar(ages.keys(), ages.values())
plt.xlabel("Age in years")
plt.ylabel("Number of Deaths")
plt.subplot(2, 3, 6)
ages = age_distribution(cured)
plt.bar(ages.keys(), ages.values())
plt.xlabel("Age in years")
plt.ylabel("Number of cured people")
plt.show()
