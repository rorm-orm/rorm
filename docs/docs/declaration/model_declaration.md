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
| `@autoCreateTime`     |   `ulong` or `DateTime` or `SysTime`    | Sets the current time <br/> on creation of the model. [More](#auto-time-fields)             |
| `@autoUpdateTime`     |   `ulong` or `DateTime` or `SysTime`    | Sets the current time <br/> on update of the model. [More](#auto-time-fields)               |
| `@choices(x)`         |           `string` or `enum`            | Sets a list of allowed values <br/> for the column. [More](#choices)                        |
| `@columnName(x)`      |                   any                   | Overwrite the default <br/> generated column name. [More](#column-name)                     |
| `@constructValue!(x)` |                   any                   | Set a constructed default value <br/> for the column. [More](#construct-value)              |
| `@defaultValue(x)`    |                   any                   | Set a constant default value <br/> for the column. [More](#default-value)                   |
| `@maxLength(x)`       |      `string` or `Nullable!string`      | Set the maximum length <br/> of the `VARCHAR` type. [More](#max-length)                     |
| `@primaryKey`         |       `integer` type or `string`        | Overwrite the default primary key. [More](#primary-key)                                     |
| `@timestamp`          |                 `ulong`                 | Set the database type <br/> to `TIMESTAMP`. [More](#timestamp)                              |
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

!!!info
    You don't need to set the `@timestamp` annotation. 
    The `ulong` will be converted to `TIMESTAMP` implicitly. 

## Choices

With the `@choices(x)` annotation, you can limit the possible values for this
field by specifying either the field type as a type of `enum T : string` or
`@choices("foo", "bar", "baz")`.

By utilizing `enum`:
```d
enum State: string
{
    ok = "ok",
    warn = "warn",
    critical = "critical",
    unknown = "unknown"
}

class User : Model
{
    State state;
}
```

or by list of `string`:
```d
class User : Model
{
    @choices("ok", "warn", "critical", "unknown")
    string state;
}
```

!!!info
    You don't need to annotate this field with `@maxLength(x)` as 
    `VARCHAR` is not used as database type.

## Column name

By setting the `@columnName(x)` annotation, you can set the column name.
If you don't set this annotation, dorm will generate the column name for you.

`x` must be of type `string`.

```d
class User : Model
{
    @columnName("admin")
    bool isAdmin
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

## Timestamp

To save an `ulong` as `TIMSTAMP` in the database, you can set the 
annotation `@timestamp`. 

```d
class User : Model
{
    @timestamp
    ulong creationTime;
}
```

!!!info
    You don't need to set this annotation if using `@autoCreateTime`
    or `@autoUpdateTime` as they set this implicit.

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
