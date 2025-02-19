package dev.surly.b;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class PersonTest {

    @Test
    void testPerson() {
        Person person = new Person("Alice", 30, "New York");
        assertEquals("Alice", person.name());
        assertEquals(30, person.age());
    }
}