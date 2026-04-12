class Animal:
    """Base class for all animals."""

    def __init__(self, name: str) -> None:
        self.name = name

    def speak(self) -> str:
        """Make the animal speak."""
        return ""


class Dog(Animal):
    """A dog."""

    def speak(self) -> str:
        return f"{self.name} says woof"
