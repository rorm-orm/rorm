# Migration File Format

## Migration File Format

The migration files utilize [TOML](https://toml.io) as format.
It is advised to read the examples of the TOML examples to get a feeling
for the options you get when using lists or tables.

TOML is used to enable developers to write migrations on their own,
in a human-readable format without the necessity to depend
on the [`makemigrations` tool](makemigrations.md).

Example migration:

```toml
[Migration]
Dependency = ""
Initial = true
Hash = "2179384940715410208"
Replaces = []

  [[Migration.Operations]]
  Type = "CreateModel"
  Name = "foo"

    [[Migration.Operations.Fields]]
    Type = "uint64"
    Name = "id"

      [[Migration.Operations.Fields.Annotations]]
      Type = "NotNull"

      [[Migration.Operations.Fields.Annotations]]
      Type = "PrimaryKey"

    [[Migration.Operations.Fields]]
    Type = "varchar"
    Name = "abc"

      [[Migration.Operations.Fields.Annotations]]
      Type = "MaxLength"
      Value = 255

      [[Migration.Operations.Fields.Annotations]]
      Type = "NotNull"

    [[Migration.Operations.Fields]]
    Type = "doubleNumber"
    Name = "def"

      [[Migration.Operations.Fields.Annotations]]
      Type = "DefaultValue"
      Value = 1.5

      [[Migration.Operations.Fields.Annotations]]
      Type = "NotNull"
```

### Migration Section

```toml
[Migration]
# Initial specifies if the migration is the initial migration.
# This has to be specified on one migration.
Initial = true

# Specify the previous migration. If the current migration is the
# initial one, this has to be an empty string.
Dependency = ""

# List of migrations this migration replaces. 
# See squashing migrations for more information about this topic.
Replaces = []

# Internal hash of the migration. This value is only used to 
# determine if it is necessary to rerun the migration creation
# process. If writing own migrations, you can just set it to "".
Hash = "123456789"

# List of operations to execute in this migration. Operations
# get executed in order. 
# 
# As TOML allows either
#
#    Operations = [{ A = "b"}]
#
# or
#
#    [[Migration.Operations]]
#    A = "b"
#
# to define lists of objects, you can use your preferred style. 
Operations = []
```

### Operations Section

The objects of the `Operations` section are a union of all possible 
database operations supported by the [`migrate` tool](migrate.md).

#### Create Model Operation

This operation will create a new table in the database.

```toml
[[Migration.Operations]]
Type = "CreateModel"

# Name is the name of the table. By convention, snake_case is used
# as format. This is enforced by the linter. 
# See linter for more information.
Name = "foo"

# This will be a column of our model
[[Migration.Operations.Fields]]
# The name of the column. Also checked by the linter.
Name = "id"

# Type of the column. Refer to Field types for a list 
# of all possible values.
#
# Note:
#   This is not the exact database type. As not all databases
#   provide the same types.
Type = "uint64"

# List of the annotation of the field with name "id"
[[Migration.Operations.Fields.Annotations]]
# Type of the annotation. Refer to Annotation Types for a complete
# list of all annotations.
Type = "PrimaryKey"

# For some annotations there is a attribute named Value required.
# If not required, it must be omitted
# Value = SomeType
```

#### Rename Model Operation

This operation will rename an existing model in the database.

```toml
[[Migration.Operations]]
Type = "RenameModel"

# Current name of the table.
Old = "foo"
# New name of the table.
New = "bar"
```

#### Delete Model Operation

This operation will delete an existing table from the database.

```toml
[[Migration.Operations]]
Type = "DeleteModel"

# Name of the table that should get deleted
Name = "foo"
```

#### Add Field Operation

This operation adds a column to an existing table.

```toml
[[Migration.Operations]]
Type = "AddField"

# Name of the table to add the column to
Name = "foo"

# The column to add to the table
[Migration.Operations.Field]
# Name of the column. Checked by the linter.
Name = "counter"

# Type of the column. Refer to Field types for a list 
# of all possible values.
#
# Note:
#   This is not the exact database type. As not all databases
#   provide the same types.
Type = "int32"

# List of annotations of the field.
# If there are no annotations, this list must be empty.
Annotations = []
```

#### Rename Field Operation

This operation renames a column from a table.

```toml
[[Migration.Operations]]
Type = "RenameField"

# Name of the table the column lives in.
TableName = "foo"

# Old name of the column
Old = "it"
# New name of the column
New = "id"
```

#### Delete Field Operation

This operation deletes a column from an existing table.

```toml
[[Migration.Operations]]
Type = "DeleteField"

# Name of the table
Name = "foo"

[Migration.Operations.Field]
# Name of the column that should be deleted
Name = "counter"
```

### Field types

### Annotation types
