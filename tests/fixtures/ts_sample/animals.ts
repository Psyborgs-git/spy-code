/** Base animal class */
export class Animal {
    constructor(public name: string) {}

    /** Make the animal speak */
    speak(): string {
        return "";
    }
}

export class Dog extends Animal {
    speak(): string {
        return `${this.name} says woof`;
    }
}
