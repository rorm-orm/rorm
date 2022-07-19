# Internal Model Representation

## Internal Model Representation

As the CLI tool for making, squashing, merging, applying and 
reverting migrations is only written once because it is able 
to utilize our [declarative migration format](migration_files.md),
for making migrations, it is required to include a step in 
your building pipeline to generate intermediate json files
that represent the current model state.

The resulting JSON should live in the root of your project
directory. The cli tools assumes the JSON file has the name
`.models.json`.

## Intermediate JSON representation

This is an example of the intermediate representation:

```json
{
  "Models": [
    {
      "Name": "foo",
      "SourceDefinedAt": {
        "File": "/path/to/source/file.rs",
        "Line": 140,
        "Column": 1
      },
      "Fields": [
        {
          "Name": "foo",
          "Type": "varchar",
          "SourceDefinedAt": {
            "File": "/path/to/source/file.rs",
            "Line": 142,
            "Column": 4
          },
          "Annotations": [
            {
              "Type": "primary_key"
            },
            {
              "Type": "not_null"
            },
            {
              "Type": "index"
            },
            {
              "Type": "max_length",
              "Value": 255,
              "Column": 4
            }
          ]
        }
      ]
    }
  ]
}
```

## Explanation

### Models

The `Name` of a model should be already in the correct table name format. 
This is enforced by the [linter](linter.md).

`SourceDefinedAt` is an optional object that specifies the file the
model originates from as well as the line number of the start of the
model definition. If the key `SourceDefinedAt` is found, `File` and `Line`
must be there as well.

`Fields` is an array of the model fields. See [Fields](#fields)

```json
{
  "Name": "table_name",
  "SourceDefinedAt": {
    "File": "/path/to/source/file.rs",
    "Line": 140,
    "Column": 1
  },
  "Fields": []
}
```

### Fields

Fields represent a column in the database.

`Name` must be in the correct column name format. For further 
information, see [linter](linter.md).

`Type` must be one of the allowed [database types](#database-types).

`SourceDefinedAt` is an optional object that specifies the file the
field originates from as well as the line number of the start of the
field definition. If the key `SourceDefinedAt` is found, `File` and `Line`
must be there as well.

`Annotation` is an array of possible [annotations](#annotations).

```json
{
  "Name": "foo",
  "Type": "varchar",
  "SourceDefinedAt": {
    "File": "/path/to/source/file.rs",
    "Line": 142,
    "Column": 4
  },
  "Annotations": []
}
```

### Annotations

Annotations must always have a key named `Type` with a possible
[annotation type](#annotation-types).

Depending on the type, it may be required to add a `Value` key.
The type of `Value` is depending on the annotation type.

E.g. `max_length` is using a value of type integer. 
`choices` on the other hand uses a value of type array of strings.

```json
{
  "Type": "max_length",
  "Value": 255
}
```

### Annotation types

|  Annotation name   |   Value required   |       Value type        |
|:------------------:|:------------------:|:-----------------------:|
| `auto_create_time` |        :x:         |                         |
| `auto_update_time` |        :x:         |                         |
|  `autoincrement`   |        :x:         |                         |
|     `choices`      | :white_check_mark: |    array of strings     |
|     `default`      | :white_check_mark: | See [default](#default) |
|      `index`       |      depends       |   See [index](#index)   |
|    `max_length`    | :white_check_mark: |         integer         |
|     `not_null`     |        :x:         |                         |
|   `primary_key`    |        :x:         |                         |
|      `unique`      |        :x:         |                         |

#### default
One of [string, number, bool].
Default types for varbinary should be encoded using hex strings. 

#### Index

If `index` is used without a value, a new index is created on the column.

If a composite index is desired, the `Name` and `Priority` fields are required:

```json
{
  "Type": "index",
  "Value":
  {
    "Name": "time-name",
    "Priority": 10
  }
}
```

The `Name` attribute is only used to determine which indexes should be 
considered as composite by checking if the `Name` is used more than once
in the same model.

The `Priority` attribute is used to determine the order in which the fields
are placed when creating the index. This can have an impact on performance.
The lower the number, the more important is the field. More significant fields
get placed first at index creation.

If two fields have the same priority, the order in the `Fields` array is used
to determine the order in the index. The order of the `Fields` array should 
map in the best case to the order of placement in the source code.

### Database types

|    Type name    | Additional notes                    |
|:---------------:|-------------------------------------|
|    `varchar`    | `max_length` annotation is required | 
|   `varbinary`   |                                     |
|     `int8`      |                                     |
|     `int16`     |                                     |
|     `int32`     |                                     |
|     `int64`     |                                     |
|     `uint8`     |                                     |
|    `uint16`     |                                     |
|    `uint32`     |                                     |
|    `uint64`     |                                     |
| `float_number`  |                                     |
| `double_number` |                                     |
|    `boolean`    |                                     |
|     `date`      |                                     |
|   `datetime`    |                                     |
|   `timestamp`   |                                     |
|     `time`      |                                     |
|    `choices`    | `choices` annotation is required    |
|      `set`      |                                     |
