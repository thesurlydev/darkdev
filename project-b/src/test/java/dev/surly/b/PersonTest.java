package dev.surly.b;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class PersonTest {

    @Test
    void testPerson() {
        Person person = new Person("Alice", 30);
        assertEquals("Alice", person.getName());
        assertEquals(30, person.getAge());
    }
}