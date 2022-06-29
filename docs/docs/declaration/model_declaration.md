# Model declaration

## Model declaration

Models are defined declarative as classes.

```d
import std.datetime.datetime;
import std.typecons;

import dorm.annotations;
import dorm.model;

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
adding another primary is possible with the `@primaryKey` annotation.

## Annotations

| Annotation              |               Allowed on                | Description                                                                                 |
|:------------------------|:---------------------------------------:|---------------------------------------------------------------------------------------------|
| `@autoCreateTime`       |          `ulong` or `SysTime`           | Sets the current time <br/> on creation of the model. [More](#auto-time-fields)             |
| `@autoUpdateTime`       |          `ulong` or `SysTime`           | Sets the current time <br/> on update of the model. [More](#auto-time-fields)               |
| `@choices(x)`           |           `string` or `enum`            | Sets a list of allowed values <br/> for the column. [More](#choices)                        |
| `@columnName(x)`        |                   any                   | Overwrite the default <br/> generated column name. [More](#column-name)                     |
| `@constructValue!(x)`   |                   any                   | Set a constructed default value <br/> for the column. [More](#construct-value)              |
| `@defaultValue(x)`      |                   any                   | Set a constant default value <br/> for the column. [More](#default-value)                   |
| `@embedded`             |                `structs`                | Embed the annotated structs <br /> in the table. [More](#embedded)                          |
| `@ignored`              |                   any                   | Ignores the annotated field. [More](#ignored)                                               |
| `@index` or `@index(x)` |                   any                   | Create an index. [More](#indexes)                                                           |
| `@maxLength(x)`         |      `string` or `Nullable!string`      | Set the maximum length <br/> of the `VARCHAR` type. [More](#max-length)                     |
| `@primaryKey`           |       `integer` type or `string`        | Overwrite the default primary key. [More](#primary-key)                                     |
| `@timestamp`            |                 `ulong`                 | Set the database type <br/> to `TIMESTAMP`. [More](#timestamp)                              |
| `@unique`               | any except ManyToMany <br/> or OneToOne | Enforce that the field value <br/> is unique throughout the column. [More](#unique)         |
| `@validator!(x)`        |                   any                   | Set a function to validate <br/> before any database operation [More](#validator-functions) |

## Auto Time fields

You can utilize the annotations `autoCreateTime` and `autoUpdateTime`
to automatically set the current time on creation respectively on update
of the model to the annotated field.

!!!info
    As `SysTime` contains timezone information, dorm will also save it in the database.

```d
class User : Model
{
    @autoCreateTime
    SysTime createdAt;

    @autoUpdateTime
    Nullable!SysTime updatedAt;
}
```

---

if you prefer using UNIX epoch instead of a SysTime field, you can 
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
    bool isAdmin;
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
    @constructValue!(() => Clock.currTime + 4.hours)
    SysTime validUntil;
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

## Embedded

The `@embedded` annotation can be used to embed the fields of the 
embedded structs in the table of the current class.

Recursively embedding structs is also supported.

```d
struct Person
{
    @maxLength(255)
    string firstName;
    
    @maxLength(255)
    string lastName;
}

class User : Model
{
    @embedded
    Person person;
}
```

This will create a table (using MySQL syntax):
```sql
CREATE TABLE user (
    id SERIAL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,  
);
```

## Ignored

The `@ignored` annotation will tell dorm to ignore the annotated field.

```d
class User : Model
{
    @ignored
    string priv;
}
```

## Indexes

The `@index(x)` annotation can be used in multiple ways.

Without `x`, an independent index is created.

```d
class User : Model
{
    @index
    uint counter;
}
```

---

To create composite indexes, specify `x` as `composite(y)`
where `y` is a string.

```d
class User : Model
{
    @index(composite("abc"))
    uint a;
    
    @index(composite("abc"))
    uint b;
}
```

!!!info
    If `@embedded` is in use, `composite(y)` can be also specified in the embedded struct.

---

To set the priority on a composite index, specify `x` as `priority(y)`,
where `y` is an `int`. 

The lower the priority, the earlier the field appears
in the definition of the index.

```d
class User : Model
{
    @index(composite("abc"), priority(2))
    uint a;
    
    @index(composite("abc"), priority(1))
    uint b;
}
```

!!!info
    If `@priority(y)` is not specified, 10 is used as default.
    On equal priorities, the field order is used to set the priorities.

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
    @validator!(x => x >= 18)
    int age;
}
```
