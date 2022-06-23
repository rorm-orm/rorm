# Model declaration

## Model declaration

Models are defined declarative as classes.

Annotations are used to 

```d
import std.datetime.datetime;
import std.typecons;

import dorm;

class User : Model
{
    @maxLength(255) // Field is limited to 255 characters
    string username;

    @maxLength(255)
    string password;
    
    @maxLength(255)
    Nullable!string email; // Nullable string field
    
    ubyte age;

    Nullable!DateTime birthday; // Nullable datetime field
}
```

will generate the following table (using MySQL syntax):

```sql
CREATE TABLE user (
    id SERIAL,
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    age TINYINT UNSIGNED NOT NULL,
    birthday DATETIME
);
```

### Additional information

- the tablename is derived from --- TODO
- column `id` is added by default by inheriting `Model`, 
adding another primary is possible via --- TODO

## Annotations

| Annotation            |               Allowed on                | Description                                                                                 |
|:----------------------|:---------------------------------------:|---------------------------------------------------------------------------------------------|
| `@autoCreateTime`     |          `ulong` or `DateTime`          | Sets the current time <br/> on creation of the model. [More](#auto-time-fields)             |
| `@autoUpdateTime`     |          `ulong` or `DateTime`          | Sets the current time <br/> on update of the model. [More](#auto-time-fields)               |
| `@constructValue!(x)` |                   any                   | Set a constructed default value <br/> for the column. [More](#construct-value)              |
| `@defaultValue(x)`    |                   any                   | Set a constant default value <br/> for the column. [More](#default-value)                   |
| `@maxLength(x)`       |      `string` or `Nullable!string`      | Set the maximum length <br/> of the `VARCHAR` type. [More](#max-length)                     |
| `@primaryKey`         |       `integer` type or `string`        | Overwrite the default primary key. [More](#primary-key)                                     |
| `@unique`             | any except ManyToMany <br/> or OneToOne | Enforce that the field value <br/> is unique throughout the column. [More](#unique)         | 
| `@validator!(x)`      |                   any                   | Set a function to validate <br/> before any database operation [More](#validator-functions) | 

## Auto Time fields

You can utilize the annotations `autoCreateTime` and `autoUpdateTime`
to automatically set the current time on creation respectively on update
of the model to the annotated field.

```d
class User : Model
{
    @autoCreateTime
    DateTime createdAt;

    @autoUpdateTime
    Nullable!DateTime updatedAt;
}
```

if you prefer using UNIX epoch instead of a DateTime field, you can 
just change the data type to `ulong`:

```d
class User : Model
{
    @autoCreateTime
    ulong createdAt;

    @autoUpdateTime
    Nullable!ulong updatedAt;
}
```

## Construct value

With the `@constructValue!(x)` annotation, you can set the default value of the field,
if omitted on creation by calling a previous defined function.

`x` is a function alias or lambda. The return value is assigned to the field on construction.

!!!caution
    `@constructValue!` functions are not serialized to the database, creating new rows
    externally, will not populate them with the desired value.

```d
import std.datetime.systime;
import std.datetime.datetime;

class User : Model
{
    @constructValue!(() => (DateTime)Clock.currTime + 4.hours)
    DateTime validUntil;
}
```

## Default value

With the `@defaultValue(x)` annotation, you can set the default value of the field,
if omitted on creation.

`x` must be of the same data type as the field.

```d
class User : Model
{
    @maxLength(255)
    @defaultValue("")
    string comment;
    
    @defaultValue(1337)
    int counter;
}
```

## Max Length

The `@maxLength(x)` annotation can (and must) be set on `string` data types.
As in most databases, `VARCHAR`, which is used as data type representation for strings,
must have set the maximum length, you must set this annotation on all `string` fields. 

```d
class User : Model
{
    @maxLength(255)
    string username;
    
    @maxLength(65536)
    Nullable!string comment;
}
```

## Primary Key

DORM adds an `id` column as primary key by default to each model.

The `@primaryKey` annotation is used to explicitly set the primary key.
The default `id` field is not added in this case.

```d
class User : Model
{
    @primaryKey
    ulong ownPrimaryKey;
}
```

## Unique

If `@unique` is set, the value must be unique throughout the complete column.

Can be used on all field types except One2One and Many2Many fields.

```d
class User : Model
{
    @unique
    int uuid;
}
```

## Validator functions

Validator functions are triggered before inserting / updating any field in the database.
Per field, you can have multiple validators. They are executed in order.

They must have the signature: `(FieldType x) => bool`.

```d
class User : Model
{
    @validator!(x => x => 18)
    int age;
}
```
