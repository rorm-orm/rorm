module dorm.utils;

import std.algorithm;
import std.array;
import std.conv;
import std.stdio;

import dorm;

/** 
 * This method can be used to generate postgres connection strings.
 *
 * There are some checks to ensure invalid characters are escapted.
 *
 * Returns: Connection string
 */
string generatePostgresConnString(ref Config conf)
{
    string[][] parameter;

    string escapeString(string str)
    {
        str = str.replace("\\", "\\\\").replace("'", "\\'");

        if (str.canFind(" "))
            str = "'" ~ str ~ "'";

        return str;
    }

    string name = escapeString(conf.name);
    string host = escapeString(conf.host);
    string port = escapeString(to!string(conf.port));
    string user = escapeString(conf.user);
    string password = escapeString(conf.password);

    parameter ~= ["dbname", name];
    parameter ~= ["host", host];
    parameter ~= ["port", port];
    parameter ~= ["user", user];
    parameter ~= ["password", password];

    return join(parameter.map!(x => join(x, "=")).array.sort.array, " ");
}

unittest
{
    import std.typecons;

    auto testCases = [
        tuple(
            Config(
                DBDriver.PostgreSQL, "name", "local", 5432, "u", "pass"
        ),
        "dbname=name host=local password=pass port=5432 user=u"

        ),
        tuple(
            Config(
                DBDriver.PostgreSQL, "test name", "host", 5432, "u", "p"
        ),
        "dbname='test name' host=host password=p port=5432 user=u"
        ),
        tuple(
            Config(
                DBDriver.PostgreSQL, "t'", "h", 5432, "u", "p"
        ),
        "dbname=t\\' host=h password=p port=5432 user=u"
        ),
        tuple(
            Config(
                DBDriver.PostgreSQL, "t '", "h", 5432, "u", "p"
        ),
        "dbname='t \\'' host=h password=p port=5432 user=u"
        ),
        tuple(
            Config(
                DBDriver.PostgreSQL, "t\\' t", "h", 5432, "u", "p"
        ),
        "dbname='t\\\\\\' t' host=h password=p port=5432 user=u"
        )
    ];

    foreach (test; testCases)
    {
        checkConf(test[0]);

        assert(generatePostgresConnString(test[0]) == test[1]);
    }
}
