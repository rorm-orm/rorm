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
  "models": [
    {
      "name": "foo",
      "source_defined_at": {
        "file": "/path/to/source/file.rs",
        "line": 140,
        "column": 1
      },
      "fields": [
        {
          "name": "foo",
          "type": "varchar",
          "source_defined_at": {
            "file": "/path/to/source/file.rs",
            "line": 142,
            "column": 4
          },
          "annotations": [
            {
              "type": "primary_key"
            },
            {
              "type": "not_null"
            },
            {
              "type": "index"
            },
            {
              "type": "max_length",
              "value": 255,
              "column": 4
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

The `name` of a model should be already in the correct table name format. 
This is enforced by the [linter](linter.md).

`source_defined_at` is an optional object that specifies the file the
model originates from as well as the line number of the start of the
model definition. If the key `source_defined_at` is found, `file` and `line`
must be there as well.

`fields` is an array of the model fields. See [Fields](#fields)

```json
{
  "name": "table_name",
  "source_defined_at": {
    "file": "/path/to/source/file.rs",
    "line": 140,
    "column": 1
  },
  "fields": []
}
```

### Fields

Fields represent a column in the database.

`name` must be in the correct column name format. For further 
information, see [linter](linter.md).

`type` must be one of the allowed [database types](#database-types).

`source_defined_at` is an optional object that specifies the file the
field originates from as well as the line number of the start of the
field definition. If the key `source_defined_at` is found, `file` and `line`
must be there as well.

`annotation` is an array of possible [annotations](#annotations).

```json
{
  "name": "foo",
  "type": "varchar",
  "source_defined_at": {
    "file": "/path/to/source/file.rs",
    "line": 142,
    "column": 4
  },
  "annotations": []
}
```

### Annotations

Annotations must always have a key named `type` with a possible
[annotation type](#annotation-types).

Depending on the type, it may be required to add a `value` key.
The type of `value` is depending on the annotation type.

E.g. `max_length` is using a value of type integer. 
`choices` on the other hand uses a value of type array of strings.

```json
{
  "type": "max_length",
  "value": 255
}
```

### Annotation types

|  Annotation name   |   Value required   |          Value type           |
|:------------------:|:------------------:|:-----------------------------:|
| `auto_create_time` |        :x:         |                               |
| `auto_update_time` |        :x:         |                               |
|     `choices`      | :white_check_mark: |       array of strings        |
|     `default`      | :white_check_mark: | one of [string, number, bool] |
|      `index`       |      depends       |      See [index](#index)      |
|    `maxLength`     | :white_check_mark: |            integer            |
|    `primaryKey`    |        :x:         |                               |
|      `unique`      |        :x:         |                               |

#### Index

If `index` is used without a value, a new index is created on the column.

If a composite index is desired, the `name` and `priority` fields are required:

```json
{
  "type": "index",
  "value":
  {
    "name": "time-name",
    "priority": 10
  }
}
```

The `name` attribute is only used to determine which indexes should be 
considered as composite by checking if the `name` is used more than once
in the same model.

The `priority` attribute is used to determine the order in which the fields
are placed when creating the index. This can have an impact on performance.
The lower the number, the more important is the field. More significant fields
get placed first at index creation.

If two fields have the same priority, the order in the `fields` array is used
to determine the order in the index. The order of the `fields` array should 
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
