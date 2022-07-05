module dorm;

import std.stdio;
import std.typecons;

import dorm.exceptions;
import dorm.utils;

import arsd.database;

/** 
 * Enum for the database driver
 */
enum DBDriver
{
    MySQL,
    SQLite,
    PostgreSQL
}

/** 
 * Config for creating a dorm instance.
 * 
 * If using DBDriver.SQLite, the param name is used to determine
 * the path of the database file. The other params beside driver
 * are not parsed in this case.
 *
 * Params: 
 *  driver   = Driver to use
 *  name     = Name of the database
 *  host     = Host of the database
 *  port     = Port to reach the database
 *  user     = Username to access the database
 *  password = Password to access the database
 */
struct Config
{
    DBDriver driver;
    string name;
    string host;
    ushort port;
    string user;
    string password;
}

/** 
 * Checks the configuration for errors
 *
 * Throws:
 *  - ConfigException
 */
void checkConf(ref Config conf)
{
    if (conf.name == "")
    {
        throw new ConfigException("Name must not be empty");
    }

    switch (conf.driver)
    {
    case DBDriver.SQLite:
        break;
    case DBDriver.MySQL:
    case DBDriver.PostgreSQL:
        if (conf.host == "")
        {
            throw new ConfigException("Host must not be empty");
        }

        if (conf.port == 0)
        {
            throw new ConfigException("Port must not be 0");
        }

        if (conf.user == "")
        {
            throw new ConfigException("Username must not be empty");
        }

        if (conf.password == "")
        {
            throw new ConfigException("Password must not be empty");
        }

        break;
    default:
        throw new ConfigException("Unknown driver type");
    }

}

/** 
 * The main API of dorm.
 */
class DB
{
    private Config conf;

    Database db;

    /** 
     * Generates the dorm instance and tries to connect to the database by
     * using the configuration struct given as parameter.
     * 
     * Params:
     *   conf = Configuration of the database connection
     * 
     * Throws: 
     *   - ConfigExcpetion
     */
    this(Config conf)
    {
        checkConf(conf);
        this.conf = conf;

        string url;
        string[string] params;

        final switch (this.conf.driver)
        {
        case DBDriver.SQLite:
            import arsd.sqlite;

            db = new Sqlite(conf.name);

            break;
        case DBDriver.MySQL:
            import arsd.mysql;

            db = new MySql(conf.host, conf.user, conf.password, conf.name);

            break;
        case DBDriver.PostgreSQL:
            import arsd.postgres;

            db = new PostgreSql(generatePostgresConnString(conf));

            break;
        }
    }
}
