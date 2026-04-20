"""Zoo module — exercises cross-file imports, calls, and type references."""

from math import add
from animals import Animal, Dog

ZOO_CAPACITY = 50


def create_zoo(size: int) -> int:
    """Create a zoo with the given size and return the total capacity."""
    return add(size, ZOO_CAPACITY)


class Zoo:
    """A zoo that contains animals."""

    def __init__(self, name: str) -> None:
        self.name = name
        self.animals = []

    def add_animal(self, animal: Animal) -> None:
        """Add an animal to the zoo."""
        self.animals.append(animal)

    def animal_count(self) -> int:
        """Return the number of animals in the zoo."""
        return add(len(self.animals), 0)

    def make_sounds(self) -> list:
        """Make all animals speak and return the list of sounds."""
        sounds = []
        for a in self.animals:
            sounds.append(a.speak())
        return sounds

    def add_dog(self, name: str) -> Dog:
        """Create and add a Dog to the zoo."""
        dog = Dog(name)
        self.add_animal(dog)
        return dog
