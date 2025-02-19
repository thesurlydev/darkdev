package dev.surly.a;

import dev.surly.b.Person;

public class HelloWorld {
    public static void main(String[] args) throws Exception {
        Person p = new Person("Shane", 40, "New York");
        System.out.printf("Hello, %s!%n", p.name());
        Thread.sleep(10000);
    }
}
