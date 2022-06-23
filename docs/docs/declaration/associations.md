# Associations

## Has One

### Polymorphism

1. Abstract classes

For polymorphism, you need a field in a Model that references 
the abstract class `Foo`.

```d
abstract class Foo : Model // no table
{
    @maxLength(255)
    string name;
}

class Bar : Foo // table "bar"
{
    int counter;
}

class Baz : Foo // table "baz"
{
    @maxLength(255)
    string password;
}

class User : Model
{
    Foo foo; // polymorphed type (user table has id + type column)
}
```

You can also use `SumType` the same way.

## Has Many

## Many To Many


