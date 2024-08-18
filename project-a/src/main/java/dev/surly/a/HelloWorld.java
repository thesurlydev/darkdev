package dev.surly.a;

import dev.surly.b.Person;

public class HelloWorld {
    public static void main(String[] args) throws Exception {
        Person p = new Person("Shane", 40);
        System.out.printf("Hello, %s!%n", p.getName());
        Thread.sleep(10000);
    }
}
