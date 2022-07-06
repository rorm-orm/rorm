import std.algorithm;
import std.array;
import std.file;
import std.format;
import std.json;
import std.process;
import std.stdio;
import std.string;

string cliDescription = "CLI tool for dorm";
string makemigrationsDescription = "Parse models and generate migration files";
string migrateDescription = "Apply migrations to database";

void displayHelp(ref string[] args)
{
    auto builder = appender!string;

    builder.put(
        format!"usage: %s"(
            [
                args[0],
                "<cmd>",
                "[-h | --help]"
            ].join(" ")
    )
    );
    builder.put("\n\n");
    builder.put(cliDescription ~ "\n" ~ "\n");

    builder.put("available commands:\n");
    builder.put("  makemigrations\t" ~ makemigrationsDescription ~ "\n");
    builder.put("  migrate\t\t" ~ migrateDescription ~ "\n");

    builder.put("\n");
    builder.put("options:\n");
    builder.put("  --help, -h\t\t" ~ "show this help message" ~ "\n");

    writeln(builder.data);
}

void displayMakemigrationsHelp(ref string[] args)
{
    auto builder = appender!string;

    builder.put(
        format!"usage: %s"(
            [
                args[0],
                "makemigrations",
                "[-h | --help | BINARY_PATH]"
            ].join(" ")
    )
    );
    builder.put("\n\n");
    builder.put(makemigrationsDescription ~ "\n");

    builder.put("\n");
    builder.put("positional options:\n");
    builder.put("  BINARY_PATH\t\t" ~ "Path to binary\n");

    builder.put("\n");
    builder.put("options:\n");
    builder.put("  --help, -h\t\t" ~ "show this help message" ~ "\n");

    writeln(builder.data);
}

void displayMigrateHelp(ref string[] args)
{

}

int main(string[] args)
{
    if (args.length == 1)
    {
        displayHelp(args);
    }
    else
    {
        switch (args[1])
        {
        case "makemigrations":

            if (args.length == 3)
            {
                if (args[2] == "-h" || args[2] == "--help")
                {
                    displayMakemigrationsHelp(args);
                    return 0;
                }
                else
                {
                    auto binPath = args[2];

                    if (!exists(binPath))
                    {
                        stderr.writeln("error: no binary found.");
                        return 1;
                    }

                    if (isDir(binPath))
                    {
                        stderr.writeln("error: found directory instead of binary.");
                        return 1;
                    }

                    // CALL TO CONVERTER
                }
            }
            else if (args.length > 3)
            {
                stderr.writeln("error: invalid argument count");
                return 1;
            }
            else
            {
                auto ret = executeShell(
                    "dub convert -f json -s 2>/dev/null",
                    null,
                    Config.none,
                    size_t.max,
                    "."
                );

                if (ret.status != 0)
                {
                    stderr.writeln("error: " ~ ret.output);
                    return 1;
                }

                try
                {
                    string cwd, bin;

                    JSONValue j = parseJSON(ret.output);
                    if (auto targetName = "targetName" in j)
                    {
                        if (targetName.type() != JSONType.string)
                        {
                            stderr.writeln("error: invalid targetName value type.");
                            return 1;
                        }

                        bin = targetName.str;
                    }
                    else
                    {
                        if (auto name = "name" in j)
                        {
                            if (name.type() != JSONType.string)
                            {
                                stderr.writeln("error: invalid name value type.");
                                return 1;
                            }

                            bin = name.str;
                        }
                        else
                        {
                            stderr.writeln("error: invalid dub.json file");
                            return 1;
                        }
                    }

                    if (auto targetPath = "targetPath" in j)
                    {
                        if (targetPath.type() != JSONType.string)
                        {
                            stderr.writeln("error: invalid targetPath value type.");
                            return 1;
                        }

                        cwd = targetPath.str;
                    }
                    else
                    {
                        cwd = getcwd();
                    }

                    string binPath = cwd ~ bin;
                    if (!exists(binPath))
                    {
                        stderr.writeln("error: no binary found.");
                        return 1;
                    }

                    if (isDir(binPath))
                    {
                        stderr.writeln("error: found directory instead of binary.");
                        return 1;
                    }

                    // CALL TO CONVERTER

                }
                catch (JSONException exc)
                {
                    stderr.writeln("error: could not parse json:\n" ~ ret.output);
                    return 1;
                }

                string binPath = join(ret.output.splitLines[$ - 3 .. $], "");

                if (!exists(binPath))
                {
                    stderr.writeln("error: no binary found.");
                    return 1;
                }

                if (isDir(binPath))
                {
                    stderr.writeln("error: found directory instead of binary.");
                    return 1;
                }

                // CALL TO CONVERTER
            }

            break;
        case "migrate":
            if (args.length == 2)
            {
                displayMakemigrationsHelp(args);
                break;
            }

            break;
        default:
            displayHelp(args);
        }
    }
    return 0;
}
