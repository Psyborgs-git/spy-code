package animals

// Animal represents a generic animal.
type Animal struct {
	Name string
}

// Speak makes the animal produce a sound.
func (a *Animal) Speak() string {
	return ""
}

// Dog is a dog.
type Dog struct {
	Animal
}

// Speak makes the dog bark.
func (d *Dog) Speak() string {
	return d.Name + " says woof"
}
