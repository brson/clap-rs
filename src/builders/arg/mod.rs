pub use self::any_arg::{AnyArg, DispOrder};
pub use self::arg::Arg;
pub use self::arg_builder::{Base, Switched, Valued, FlagBuilder, OptBuilder, PosBuilder};
pub use self::arg_matcher::ArgMatcher;
pub use self::arg_matches::{Values, OsValues, ArgMatches};
pub use self::group::ArgGroup;
pub use self::matched_arg::MatchedArg;
pub use self::settings::{ArgFlags, ArgSettings};
pub use self::subcommand::SubCommand;
pub use self::flag::FlagBuilder;
pub use self::option::OptBuilder;
pub use self::positional::PosBuilder;
pub use self::base::Base;
pub use self::switched::Switched;
pub use self::valued::Valued;

#[macro_use]
mod macros;
mod arg;
pub mod any_arg;
mod arg_matches;
mod arg_matcher;
mod subcommand;
mod arg_builder;
mod matched_arg;
mod group;
pub mod settings;

mod flag;
mod positional;
mod option;
mod base;
mod valued;
mod switched;

#[cfg(feature = "yaml")]
use std::collections::BTreeMap;
use std::rc::Rc;
use std::ffi::{OsString, OsStr};
#[cfg(target_os="windows")]
use osstringext::OsStrExt3;
#[cfg(not(target_os="windows"))]
use std::os::unix::ffi::OsStrExt;


#[cfg(feature = "yaml")]
use yaml_rust::Yaml;
use vec_map::VecMap;

use usage_parser::UsageParser;
use args::settings::ArgSettings;
use args::arg_builder::{Base, Valued, Switched};

/// The abstract representation of a command line argument. Used to set all the options and
/// relationships that define a valid argument for the program.
///
/// There are two methods for constructing [`Arg`]s, using the builder pattern and setting options
/// manually, or using a usage string which is far less verbose but has fewer options. You can also
/// use a combination of the two methods to achieve the best of both worlds.
///
/// # Examples
///
/// ```rust
/// # use clap::Arg;
/// // Using the traditional builder pattern and setting each option manually
/// let cfg = Arg::with_name("config")
///       .short("c")
///       .long("config")
///       .takes_value(true)
///       .value_name("FILE")
///       .help("Provides a config file to myprog");
/// // Using a usage string (setting a similar argument to the one above)
/// let input = Arg::from_usage("-i, --input=[FILE] 'Provides an input file to the program'");
/// ```
/// [`Arg`]: ./struct.Arg.html
#[allow(missing_debug_implementations)]
#[derive(Default, Clone)]
pub struct Arg<'key, 'other>
    where 'key: 'other
{
    #[doc(hidden)]
    pub b: Base<'key, 'other>,
    #[doc(hidden)]
    pub s: Switched<'other>,
    #[doc(hidden)]
    pub v: Valued<'key, 'other>,
    #[doc(hidden)]
    pub index: Option<u64>,
    #[doc(hidden)]
    pub r_ifs: Option<Vec<(&'key str, &'other str)>>,
}

impl<'key, 'other> Arg<'key, 'other> {
    /// Creates a new instance of [`Arg`] using a unique string name. The name will be used to get
    /// information about whether or not the argument was used at runtime, get values, set
    /// relationships with other args, etc..
    ///
    /// **NOTE:** In the case of arguments that take values (i.e. [`Arg::takes_value(true)`])
    /// and positional arguments (i.e. those without a preceding `-` or `--`) the name will also
    /// be displayed when the user prints the usage/help information of the program.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    /// # ;
    /// ```
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg`]: ./struct.Arg.html
    pub fn with_name(n: &'key str) -> Self { Arg { b: Base::new(n), ..Default::default() } }

    /// Creates a new instance of [`Arg`] from a .yml (YAML) file.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # #[macro_use]
    /// # extern crate clap;
    /// # use clap::Arg;
    /// # fn main() {
    /// let yml = load_yaml!("arg.yml");
    /// let arg = Arg::from_yaml(yml);
    /// # }
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    #[cfg(feature = "yaml")]
    pub fn from_yaml(y: &BTreeMap<Yaml, Yaml>) -> Arg {
        // We WANT this to panic on error...so expect() is good.
        let name_yml = y.keys().nth(0).unwrap();
        let name_str = name_yml.as_str().unwrap();
        let mut a = Arg::with_name(name_str);
        let arg_settings = y.get(name_yml).unwrap().as_hash().unwrap();

        for (k, v) in arg_settings.iter() {
            a = match k.as_str().unwrap() {
                "short" => yaml_to_str!(a, v, short),
                "long" => yaml_to_str!(a, v, long),
                "aliases" => yaml_vec_or_str!(v, a, alias),
                "help" => yaml_to_str!(a, v, help),
                "long_help" => yaml_to_str!(a, v, long_help),
                "required" => yaml_to_bool!(a, v, required),
                "required_if" => yaml_tuple2!(a, v, required_if),
                "required_ifs" => yaml_tuple2!(a, v, required_if),
                "takes_value" => yaml_to_bool!(a, v, takes_value),
                "index" => yaml_to_u64!(a, v, index),
                "global" => yaml_to_bool!(a, v, global),
                "multiple" => yaml_to_bool!(a, v, multiple),
                "hidden" => yaml_to_bool!(a, v, hidden),
                "next_line_help" => yaml_to_bool!(a, v, next_line_help),
                "empty_values" => yaml_to_bool!(a, v, empty_values),
                "group" => yaml_to_str!(a, v, group),
                "number_of_values" => yaml_to_u64!(a, v, number_of_values),
                "max_values" => yaml_to_u64!(a, v, max_values),
                "min_values" => yaml_to_u64!(a, v, min_values),
                "value_name" => yaml_to_str!(a, v, value_name),
                "use_delimiter" => yaml_to_bool!(a, v, use_delimiter),
                "allow_hyphen_values" => yaml_to_bool!(a, v, allow_hyphen_values),
                "require_delimiter" => yaml_to_bool!(a, v, require_delimiter),
                "value_delimiter" => yaml_to_str!(a, v, value_delimiter),
                "required_unless" => yaml_to_str!(a, v, required_unless),
                "display_order" => yaml_to_usize!(a, v, display_order),
                "default_value" => yaml_to_str!(a, v, default_value),
                "default_value_if" => yaml_tuple3!(a, v, default_value_if),
                "default_value_ifs" => yaml_tuple3!(a, v, default_value_if),
                "value_names" => yaml_vec_or_str!(v, a, value_name),
                "groups" => yaml_vec_or_str!(v, a, group),
                "requires" => yaml_vec_or_str!(v, a, requires),
                "requires_if" => yaml_tuple2!(a, v, requires_if),
                "requires_ifs" => yaml_tuple2!(a, v, requires_if),
                "conflicts_with" => yaml_vec_or_str!(v, a, conflicts_with),
                "overrides_with" => yaml_vec_or_str!(v, a, overrides_with),
                "possible_values" => yaml_vec_or_str!(v, a, possible_value),
                "required_unless_one" => yaml_vec_or_str!(v, a, required_unless),
                "required_unless_all" => {
                    a = yaml_vec_or_str!(v, a, required_unless);
                    a.setb(ArgSettings::RequiredUnlessAll);
                    a
                }
                s => {
                    panic!("Unknown Arg setting '{}' in YAML file for arg '{}'",
                           s,
                           name_str)
                }
            }
        }

        a
    }

    /// Creates a new instance of [`Arg`] from a usage string. Allows creation of basic settings
    /// for the [`Arg`]. The syntax is flexible, but there are some rules to follow.
    ///
    /// **NOTE**: Not all settings may be set using the usage string method. Some properties are
    /// only available via the builder pattern.
    ///
    /// **NOTE**: Only ASCII values are officially supported in [`Arg::from_usage`] strings. Some
    /// UTF-8 codepoints may work just fine, but this is not guaranteed.
    ///
    /// # Syntax
    ///
    /// Usage strings typically following the form:
    ///
    /// ```notrust
    /// [explicit name] [short] [long] [value names] [help string]
    /// ```
    ///
    /// This is not a hard rule as the attributes can appear in other orders. There are also
    /// several additional sigils which denote additional settings. Below are the details of each
    /// portion of the string.
    ///
    /// ### Explicit Name
    ///
    /// This is an optional field, if it's omitted the argument will use one of the additional
    /// fields as the name using the following priority order:
    ///
    ///  * Explicit Name (This always takes precedence when present)
    ///  * Long
    ///  * Short
    ///  * Value Name
    ///
    /// `clap` determines explicit names as the first string of characters between either `[]` or
    /// `<>` where `[]` has the dual notation of meaning the argument is optional, and `<>` meaning
    /// the argument is required.
    ///
    /// Explicit names may be followed by:
    ///  * The multiple denotation `...`
    ///
    /// Example explicit names as follows (`ename` for an optional argument, and `rname` for a
    /// required argument):
    ///
    /// ```notrust
    /// [ename] -s, --long 'some flag'
    /// <rname> -r, --longer 'some other flag'
    /// ```
    ///
    /// ### Short
    ///
    /// This is set by placing a single character after a leading `-`.
    ///
    /// Shorts may be followed by
    ///  * The multiple denotation `...`
    ///  * An optional comma `,` which is cosmetic only
    ///  * Value notation
    ///
    /// Example shorts are as follows (`-s`, and `-r`):
    ///
    /// ```notrust
    /// -s, --long 'some flag'
    /// <rname> -r [val], --longer 'some option'
    /// ```
    ///
    /// ### Long
    ///
    /// This is set by placing a word (no spaces) after a leading `--`.
    ///
    /// Shorts may be followed by
    ///  * The multiple denotation `...`
    ///  * Value notation
    ///
    /// Example longs are as follows (`--some`, and `--rapid`):
    ///
    /// ```notrust
    /// -s, --some 'some flag'
    /// --rapid=[FILE] 'some option'
    /// ```
    ///
    /// ### Values (Value Notation)
    ///
    /// This is set by placing a word(s) between `[]` or `<>` optionally after `=` (although this
    /// is cosmetic only and does not affect functionality). If an explicit name has **not** been
    /// set, using `<>` will denote a required argument, and `[]` will denote an optional argument
    ///
    /// Values may be followed by
    ///  * The multiple denotation `...`
    ///  * More Value notation
    ///
    /// More than one value will also implicitly set the arguments number of values, i.e. having
    /// two values, `--option [val1] [val2]` specifies that in order for option to be satisified it
    /// must receive exactly two values
    ///
    /// Example values are as follows (`FILE`, and `SPEED`):
    ///
    /// ```notrust
    /// -s, --some [FILE] 'some option'
    /// --rapid=<SPEED>... 'some required multiple option'
    /// ```
    ///
    /// ### Help String
    ///
    /// The help string is denoted between a pair of single quotes `''` and may contain any
    /// characters.
    ///
    /// Example help strings are as follows:
    ///
    /// ```notrust
    /// -s, --some [FILE] 'some option'
    /// --rapid=<SPEED>... 'some required multiple option'
    /// ```
    ///
    /// ### Additional Sigils
    ///
    /// Multiple notation `...` (three consecutive dots/periods) specifies that this argument may
    /// be used multiple times. Do not confuse multiple occurrences (`...`) with multiple values.
    /// `--option val1 val2` is a single occurrence with multiple values. `--flag --flag` is
    /// multiple occurrences (and then you can obviously have instances of both as well)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// App::new("prog")
    ///     .args(&[
    ///         Arg::from_usage("--config <FILE> 'key required file for the configuration and no short'"),
    ///         Arg::from_usage("-d, --debug... 'turns on debugging information and allows multiples'"),
    ///         Arg::from_usage("[input] 'keyn optional input file to use'")
    /// ])
    /// # ;
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    /// [`Arg::from_usage`]: ./struct.Arg.html#method.from_usage
    pub fn from_usage(u: &'key str) -> Self {
        let parser = UsageParser::from_usage(u);
        parser.parse()
    }

    /// Sets the short version of the argument without the preceding `-`.
    ///
    /// By default `clap` automatically assigns `V` and `h` to the auto-generated `version` and
    /// `help` arguments respectively. You may use the uppercase `V` or lowercase `h` for your own
    /// arguments, in which case `clap` simply will not assign those to the auto-generated
    /// `version` or `help` arguments.
    ///
    /// **NOTE:** Any leading `-` characters will be stripped, and only the first
    /// non `-` character will be used as the [`short`] version
    ///
    /// # Examples
    ///
    /// To set [`short`] use a single valid UTF-8 code point. If you supply a leading `-` such as
    /// `-c`, the `-` will be stripped.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .short("c")
    /// # ;
    /// ```
    ///
    /// Setting [`short`] allows using the argument via a single hyphen (`-`) such as `-c`
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("config")
    ///         .short("c"))
    ///     .get_matches_from(vec![
    ///         "prog", "-c"
    ///     ]);
    ///
    /// assert!(m.is_present("config"));
    /// ```
    /// [`short`]: ./struct.Arg.html#method.short
    pub fn short<S: AsRef<str>>(mut self, s: S) -> Self {
        self.s.short = s.as_ref().trim_left_matches(|c| c == '-').chars().nth(0);
        self
    }

    /// Sets the long version of the argument without the preceding `--`.
    ///
    /// By default `clap` automatically assigns `version` and `help` to the auto-generated
    /// `version` and `help` arguments respectively. You may use the word `version` or `help` for
    /// the long form of your own arguments, in which case `clap` simply will not assign those to
    /// the auto-generated `version` or `help` arguments.
    ///
    /// **NOTE:** Any leading `-` characters will be stripped
    ///
    /// # Examples
    ///
    /// To set `long` use a word containing valid UTF-8 codepoints. If you supply a double leading
    /// `--` such as `--config` they will be stripped. Hyphens in the middle of the word, however,
    /// will *not* be stripped (i.e. `config-file` is allowed)
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("cfg")
    ///     .long("config")
    /// # ;
    /// ```
    ///
    /// Setting `long` allows using the argument via a double hyphen (`--`) such as `--config`
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config"))
    ///     .get_matches_from(vec![
    ///         "prog", "--config"
    ///     ]);
    ///
    /// assert!(m.is_present("cfg"));
    /// ```
    pub fn long(mut self, l: &'other str) -> Self {
        self.s.long = Some(l.trim_left_matches(|c| c == '-'));
        self
    }

    /// Allows adding a [`Arg`] alias, which function as "hidden" arguments that
    /// automatically dispatch as if this argument was used. This is more efficient, and easier
    /// than creating multiple hidden arguments as one only needs to check for the existence of
    /// this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///             .arg(Arg::with_name("test")
    ///             .long("test")
    ///             .alias("alias")
    ///             .takes_value(true))
    ///        .get_matches_from(vec![
    ///             "prog", "--alias", "cool"
    ///         ]);
    /// assert!(m.is_present("test"));
    /// assert_eq!(m.value_of("test"), Some("cool"));
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    pub fn alias<S: Into<&'other str>>(mut self, name: S) -> Self {
        if let Some(ref mut als) = self.s.aliases {
            als.push((name.into(), false));
        } else {
            self.s.aliases = Some(vec![(name.into(), false)]);
        }
        self
    }

    /// Allows adding [`Arg`] aliases, which function as "hidden" arguments that
    /// automatically dispatch as if this argument was used. This is more efficient, and easier
    /// than creating multiple hidden subcommands as one only needs to check for the existence of
    /// this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///             .arg(Arg::with_name("test")
    ///                     .long("test")
    ///                     .aliases(&["do-stuff", "do-tests", "tests"])
    ///                     .help("the file to add")
    ///                     .required(false))
    ///             .get_matches_from(vec![
    ///                 "prog", "--do-tests"
    ///             ]);
    /// assert!(m.is_present("test"));
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    pub fn aliases(mut self, names: &[&'other str]) -> Self {
        if let Some(ref mut als) = self.s.aliases {
            for n in names {
                als.push((n, false));
            }
        } else {
            self.s.aliases = Some(names.iter().map(|n| (*n, false)).collect::<Vec<_>>());
        }
        self
    }

    /// Allows adding a [`Arg`] alias that functions exactly like those defined with
    /// [`Arg::alias`], except that they are visible inside the help message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///             .arg(Arg::with_name("test")
    ///                 .visible_alias("something-awesome")
    ///                 .long("test")
    ///                 .takes_value(true))
    ///        .get_matches_from(vec![
    ///             "prog", "--something-awesome", "coffee"
    ///         ]);
    /// assert!(m.is_present("test"));
    /// assert_eq!(m.value_of("test"), Some("coffee"));
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    /// [`App::alias`]: ./struct.Arg.html#method.alias
    pub fn visible_alias<S: Into<&'other str>>(mut self, name: S) -> Self {
        if let Some(ref mut als) = self.s.aliases {
            als.push((name.into(), true));
        } else {
            self.s.aliases = Some(vec![(name.into(), true)]);
        }
        self
    }

    /// Allows adding multiple [`Arg`] aliases that functions exactly like those defined
    /// with [`Arg::aliases`], except that they are visible inside the help message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///             .arg(Arg::with_name("test")
    ///                 .long("test")
    ///                 .visible_aliases(&["something", "awesome", "cool"]))
    ///        .get_matches_from(vec![
    ///             "prog", "--awesome"
    ///         ]);
    /// assert!(m.is_present("test"));
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    /// [`App::aliases`]: ./struct.Arg.html#method.aliases
    pub fn visible_aliases(mut self, names: &[&'other str]) -> Self {
        if let Some(ref mut als) = self.s.aliases {
            for n in names {
                als.push((n, true));
            }
        } else {
            self.s.aliases = Some(names.iter().map(|n| (*n, true)).collect::<Vec<_>>());
        }
        self
    }

    /// Sets the short help text of the argument that will be displayed to the user when they print
    /// the help information with `-h`. Typically, this is a short (one line) description of the
    /// arg.
    ///
    /// **NOTE:** If only `Arg::help` is provided, and not [`Arg::long_help`] but the user requests
    /// `--help` clap will still display the contents of `help` appropriately
    ///
    /// **NOTE:** Only `Arg::help` is used in completion script generation in order to be concise
    ///
    /// # Examples
    ///
    /// Any valid UTF-8 is allowed in the help text. The one exception is when one wishes to
    /// include a newline in the help text and have the following text be properly aligned with all
    /// the other help text.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .help("The config file used by the myprog")
    /// # ;
    /// ```
    ///
    /// Setting `help` displays a short message to the side of the argument when the user passes
    /// `-h` or `--help` (by default).
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .help("Some help text describing the --config arg"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    ///     --config     Some help text describing the --config arg
    /// -h, --help       Prints help information
    /// -V, --version    Prints version information
    /// ```
    /// [`Arg::long_help`]: ./struct.Arg.html#method.long_help
    pub fn help(mut self, h: &'other str) -> Self {
        self.b.help = Some(h);
        self
    }

    /// Sets the long help text of the argument that will be displayed to the user when they print
    /// the help information with `--help`. Typically this a more detailed (multi-line) message
    /// that describes the arg.
    ///
    /// **NOTE:** If only `long_help` is provided, and not [`Arg::help`] but the user requests `-h`
    /// clap will still display the contents of `long_help` appropriately
    ///
    /// **NOTE:** Only [`Arg::help`] is used in completion script generation in order to be concise
    ///
    /// # Examples
    ///
    /// Any valid UTF-8 is allowed in the help text. The one exception is when one wishes to
    /// include a newline in the help text and have the following text be properly aligned with all
    /// the other help text.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .long_help(
    /// "The config file used by the myprog must be in JSON format
    /// with only valid keys and may not contain other nonsense
    /// that cannot be read by this program. Obviously I'm going on
    /// and on, so I'll stop now.")
    /// # ;
    /// ```
    ///
    /// Setting `help` displays a short message to the side of the argument when the user passes
    /// `-h` or `--help` (by default).
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .long_help(
    /// "The config file used by the myprog must be in JSON format
    /// with only valid keys and may not contain other nonsense
    /// that cannot be read by this program. Obviously I'm going on
    /// and on, so I'll stop now."))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    ///    --config
    ///         The config file used by the myprog must be in JSON format
    ///         with only valid keys and may not contain other nonsense
    ///         that cannot be read by this program. Obviously I'm going on
    ///         and on, so I'll stop now.
    ///
    /// -h, --help       
    ///         Prints help information
    ///
    /// -V, --version    
    ///         Prints version information
    /// ```
    /// [`Arg::help`]: ./struct.Arg.html#method.help
    pub fn long_help(mut self, h: &'other str) -> Self {
        self.b.long_help = Some(h);
        self
    }

    /// Sets an arg that override this arg's required setting. (i.e. this arg will be required
    /// unless this other argument is present).
    ///
    /// **Pro Tip:** Using [`Arg::required_unless`] implies [`Arg::required`] and is therefore not
    /// mandatory to also set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_unless("debug")
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_unless(name)`] requires that the argument be used at runtime
    /// *unless* `name` is present. In the following example, the required argument is *not*
    /// provided, but it's not an error because the `unless` arg has been supplied.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless("dbg")
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--debug"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Setting [`Arg::required_unless(name)`] and *not* supplying `name` or this arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless("dbg")
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::required_unless`]: ./struct.Arg.html#method.required_unless
    /// [`Arg::required`]: ./struct.Arg.html#method.required
    /// [`Arg::required_unless(name)`]: ./struct.Arg.html#method.required_unless
    pub fn required_unless(mut self, name: &'key str) -> Self {
        if let Some(ref mut vec) = self.b.r_unless {
            vec.push(name);
        } else {
            self.b.r_unless = Some(vec![name]);
        }
        self.required(true)
    }

    /// Sets args that override this arg's required setting. (i.e. this arg will be required unless
    /// all these other arguments are present).
    ///
    /// **NOTE:** If you wish for this argument to only be required if *one of* these args are
    /// present see [`Arg::required_unless_one`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_unless_all(&["cfg", "dbg"])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_unless_all(names)`] requires that the argument be used at runtime
    /// *unless* *all* the args in `names` are present. In the following example, the required
    /// argument is *not* provided, but it's not an error because all the `unless` args have been
    /// supplied.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless_all(&["dbg", "infile"])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("infile")
    ///         .short("i")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--debug", "-i", "file"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Setting [`Arg::required_unless_all(names)`] and *not* supplying *all* of `names` or this
    /// arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless_all(&["dbg", "infile"])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("infile")
    ///         .short("i")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::required_unless_one`]: ./struct.Arg.html#method.required_unless_one
    /// [`Arg::required_unless_all(names)`]: ./struct.Arg.html#method.required_unless_all
    pub fn required_unless_all(mut self, names: &[&'key str]) -> Self {
        if let Some(ref mut vec) = self.b.r_unless {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.r_unless = Some(names.iter().map(|s| *s).collect::<Vec<_>>());
        }
        self.setb(ArgSettings::RequiredUnlessAll);
        self.required(true)
    }

    /// Sets args that override this arg's [required] setting. (i.e. this arg will be required
    /// unless *at least one of* these other arguments are present).
    ///
    /// **NOTE:** If you wish for this argument to only be required if *all of* these args are
    /// present see [`Arg::required_unless_all`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_unless_all(&["cfg", "dbg"])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_unless_one(names)`] requires that the argument be used at runtime
    /// *unless* *at least one of* the args in `names` are present. In the following example, the
    /// required argument is *not* provided, but it's not an error because one the `unless` args
    /// have been supplied.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless_one(&["dbg", "infile"])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("infile")
    ///         .short("i")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--debug"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Setting [`Arg::required_unless_one(names)`] and *not* supplying *at least one of* `names`
    /// or this arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless_one(&["dbg", "infile"])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("infile")
    ///         .short("i")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [required]: ./struct.Arg.html#method.required
    /// [`Arg::required_unless_one(names)`]: ./struct.Arg.html#method.required_unless_one
    /// [`Arg::required_unless_all`]: ./struct.Arg.html#method.required_unless_all
    pub fn required_unless_one(mut self, names: &[&'key str]) -> Self {
        if let Some(ref mut vec) = self.b.r_unless {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.r_unless = Some(names.iter().map(|s| *s).collect::<Vec<_>>());
        }
        self.required(true)
    }

    /// Sets a conflicting argument by name. I.e. when using this argument,
    /// the following argument can't be present and vice versa.
    ///
    /// **NOTE:** Conflicting rules take precedence over being required by default. Conflict rules
    /// only need to be set for one of the two arguments, they do not need to be set for each.
    ///
    /// **NOTE:** Defining a conflict is two-way, but does *not* need to defined for both arguments
    /// (i.e. if A conflicts with B, defining A.conflicts_with(B) is sufficient. You do not need
    /// need to also do B.conflicts_with(A))
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .conflicts_with("debug")
    /// # ;
    /// ```
    ///
    /// Setting conflicting argument, and having both arguments present at runtime is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .conflicts_with("debug")
    ///         .long("config"))
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--debug", "--config", "file.conf"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::ArgumentConflict);
    /// ```
    pub fn conflicts_with(mut self, name: &'key str) -> Self {
        if let Some(ref mut vec) = self.b.blacklist {
            vec.push(name);
        } else {
            self.b.blacklist = Some(vec![name]);
        }
        self
    }

    /// The same as [`Arg::conflicts_with`] but allows specifying multiple two-way conlicts per
    /// argument.
    ///
    /// **NOTE:** Conflicting rules take precedence over being required by default. Conflict rules
    /// only need to be set for one of the two arguments, they do not need to be set for each.
    ///
    /// **NOTE:** Defining a conflict is two-way, but does *not* need to defined for both arguments
    /// (i.e. if A conflicts with B, defining A.conflicts_with(B) is sufficient. You do not need
    /// need to also do B.conflicts_with(A))
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .conflicts_with_all(&["debug", "input"])
    /// # ;
    /// ```
    ///
    /// Setting conflicting argument, and having any of the arguments present at runtime with a
    /// conflicting argument is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .conflicts_with_all(&["debug", "input"])
    ///         .long("config"))
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "file.conf", "file.txt"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::ArgumentConflict);
    /// ```
    /// [`Arg::conflicts_with`]: ./struct.Arg.html#method.conflicts_with
    pub fn conflicts_with_all(mut self, names: &[&'key str]) -> Self {
        if let Some(ref mut vec) = self.b.blacklist {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.blacklist = Some(names.iter().map(|s| *s).collect::<Vec<_>>());
        }
        self
    }

    /// Sets a overridable argument by name. I.e. this argument and the following argument
    /// will override each other in POSIX style (whichever argument was specified at runtime
    /// **last** "wins")
    ///
    /// **NOTE:** When an argument is overridden it is essentially as if it never was used, any
    /// conflicts, requirements, etc. are evaluated **after** all "overrides" have been removed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::from_usage("-f, --flag 'some flag'")
    ///         .conflicts_with("debug"))
    ///     .arg(Arg::from_usage("-d, --debug 'other flag'"))
    ///     .arg(Arg::from_usage("-c, --color 'third flag'")
    ///         .overrides_with("flag"))
    ///     .get_matches_from(vec![
    ///         "prog", "-f", "-d", "-c"]);
    ///             //    ^~~~~~~~~~~~^~~~~ flag is overridden by color
    ///
    /// assert!(m.is_present("color"));
    /// assert!(m.is_present("debug")); // even though flag conflicts with debug, it's as if flag
    ///                                 // was never used because it was overridden with color
    /// assert!(!m.is_present("flag"));
    /// ```
    pub fn overrides_with(mut self, name: &'key str) -> Self {
        if let Some(ref mut vec) = self.b.overrides {
            vec.push(name.as_ref());
        } else {
            self.b.overrides = Some(vec![name.as_ref()]);
        }
        self
    }

    /// Sets multiple mutually overridable arguments by name. I.e. this argument and the following
    /// argument will override each other in POSIX style (whichever argument was specified at
    /// runtime **last** "wins")
    ///
    /// **NOTE:** When an argument is overridden it is essentially as if it never was used, any
    /// conflicts, requirements, etc. are evaluated **after** all "overrides" have been removed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::from_usage("-f, --flag 'some flag'")
    ///         .conflicts_with("color"))
    ///     .arg(Arg::from_usage("-d, --debug 'other flag'"))
    ///     .arg(Arg::from_usage("-c, --color 'third flag'")
    ///         .overrides_with_all(&["flag", "debug"]))
    ///     .get_matches_from(vec![
    ///         "prog", "-f", "-d", "-c"]);
    ///             //    ^~~~~~^~~~~~~~~ flag and debug are overridden by color
    ///
    /// assert!(m.is_present("color")); // even though flag conflicts with color, it's as if flag
    ///                                 // and debug were never used because they were overridden
    ///                                 // with color
    /// assert!(!m.is_present("debug"));
    /// assert!(!m.is_present("flag"));
    /// ```
    pub fn overrides_with_all(mut self, names: &[&'key str]) -> Self {
        if let Some(ref mut vec) = self.b.overrides {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.overrides = Some(names.iter().map(|s| *s).collect::<Vec<_>>());
        }
        self
    }

    /// Sets an argument by name that is required when this one is present I.e. when
    /// using this argument, the following argument *must* be present.
    ///
    /// **NOTE:** [Conflicting] rules and [override] rules take precedence over being required
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .requires("input")
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::requires(name)`] requires that the argument be used at runtime if the
    /// defining argument is used. If the defining argument isn't used, the other argument isn't
    /// required
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires("input")
    ///         .long("config"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use cfg, so input wasn't required
    /// ```
    ///
    /// Setting [`Arg::requires(name)`] and *not* supplying that argument is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires("input")
    ///         .long("config"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "file.conf"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [override]: ./struct.Arg.html#method.overrides_with
    pub fn requires(mut self, name: &'key str) -> Self {
        if let Some(ref mut vec) = self.b.requires {
            vec.push((None, name));
        } else {
            let mut vec = vec![];
            vec.push((None, name));
            self.b.requires = Some(vec);
        }
        self
    }

    /// Allows a conditional requirement. The requirement will only become valid if this arg's value
    /// equals `val`.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// requires_if:
    ///     - [val, arg]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .requires_if("val", "arg")
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::requires_if(val, arg)`] requires that the `arg` be used at runtime if the
    /// defining argument's value is equal to `val`. If the defining argument is anything other than
    /// `val`, the other argument isn't required.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires_if("my.cfg", "other")
    ///         .long("config"))
    ///     .arg(Arg::with_name("other"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "some.cfg"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use --config=my.cfg, so other wasn't required
    /// ```
    ///
    /// Setting [`Arg::requires_if(val, arg)`] and setting the value to `val` but *not* supplying
    /// `arg` is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires_if("my.cfg", "input")
    ///         .long("config"))
    ///     .arg(Arg::with_name("input"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "my.cfg"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [override]: ./struct.Arg.html#method.overrides_with
    pub fn requires_if(mut self, val: &'other str, arg: &'key str) -> Self {
        if let Some(ref mut vec) = self.b.requires {
            vec.push((Some(val), arg));
        } else {
            self.b.requires = Some(vec![(Some(val), arg)]);
        }
        self
    }

    /// Allows multiple conditional requirements. The requirement will only become valid if this arg's value
    /// equals `val`.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// requires_if:
    ///     - [val, arg]
    ///     - [val2, arg2]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .requires_ifs(&[
    ///         ("val", "arg"),
    ///         ("other_val", "arg2"),
    ///     ])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::requires_ifs(&["val", "arg"])`] requires that the `arg` be used at runtime if the
    /// defining argument's value is equal to `val`. If the defining argument's value is anything other
    /// than `val`, `arg` isn't required.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires_ifs(&[
    ///             ("special.conf", "opt"),
    ///             ("other.conf", "other"),
    ///         ])
    ///         .long("config"))
    ///     .arg(Arg::with_name("opt")
    ///         .long("option")
    ///         .takes_value(true))
    ///     .arg(Arg::with_name("other"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "special.conf"
    ///     ]);
    ///
    /// assert!(res.is_err()); // We  used --config=special.conf so --option <val> is required
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [override]: ./struct.Arg.html#method.overrides_with
    pub fn requires_ifs(mut self, ifs: &[(&'other str, &'key str)]) -> Self {
        if let Some(ref mut vec) = self.b.requires {
            for &(val, arg) in ifs {
                vec.push((Some(val), arg));
            }
        } else {
            let mut vec = vec![];
            for &(val, arg) in ifs {
                vec.push((Some(val), arg));
            }
            self.b.requires = Some(vec);
        }
        self
    }

    /// Allows specifying that an argument is [required] conditionally. The requirement will only
    /// become valid if the specified `arg`'s value equals `val`.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// required_if:
    ///     - [arg, val]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_if("other_arg", "value")
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_if(arg, val)`] makes this arg required if the `arg` is used at
    /// runtime and it's value is equal to `val`. If the `arg`'s value is anything other than `val`,
    /// this argument isn't required.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .required_if("other", "special")
    ///         .long("config"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--other", "not-special"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use --other=special, so "cfg" wasn't required
    /// ```
    ///
    /// Setting [`Arg::required_if(arg, val)`] and having `arg` used with a vaue of `val` but *not*
    /// using this arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .required_if("other", "special")
    ///         .long("config"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--other", "special"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [required]: ./struct.Arg.html#method.required
    pub fn required_if(mut self, arg: &'key str, val: &'other str) -> Self {
        if let Some(ref mut vec) = self.r_ifs {
            vec.push((arg, val));
        } else {
            self.r_ifs = Some(vec![(arg, val)]);
        }
        self
    }

    /// Allows specifying that an argument is [required] based on multiple conditions. The
    /// conditions are set up in a `(arg, val)` style tuple. The requirement will only become valid
    /// if one of the specified `arg`'s value equals it's corresponding `val`.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// required_if:
    ///     - [arg, val]
    ///     - [arg2, val2]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_ifs(&[
    ///         ("extra", "val"),
    ///         ("option", "spec")
    ///     ])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_ifs(&[(arg, val)])`] makes this arg required if any of the `arg`s
    /// are used at runtime and it's corresponding value is equal to `val`. If the `arg`'s value is
    /// anything other than `val`, this argument isn't required.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_ifs(&[
    ///             ("extra", "val"),
    ///             ("option", "spec")
    ///         ])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("extra")
    ///         .takes_value(true)
    ///         .long("extra"))
    ///     .arg(Arg::with_name("option")
    ///         .takes_value(true)
    ///         .long("option"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--option", "other"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use --option=spec, or --extra=val so "cfg" isn't required
    /// ```
    ///
    /// Setting [`Arg::required_ifs(&[(arg, val)])`] and having any of the `arg`s used with it's
    /// vaue of `val` but *not* using this arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_ifs(&[
    ///             ("extra", "val"),
    ///             ("option", "spec")
    ///         ])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("extra")
    ///         .takes_value(true)
    ///         .long("extra"))
    ///     .arg(Arg::with_name("option")
    ///         .takes_value(true)
    ///         .long("option"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--option", "spec"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [required]: ./struct.Arg.html#method.required
    pub fn required_ifs(mut self, ifs: &[(&'key str, &'other str)]) -> Self {
        if let Some(ref mut vec) = self.r_ifs {
            for r_if in ifs {
                vec.push((r_if.0, r_if.1));
            }
        } else {
            let mut vec = vec![];
            for r_if in ifs {
                vec.push((r_if.0, r_if.1));
            }
            self.r_ifs = Some(vec);
        }
        self
    }

    /// Sets multiple arguments by names that are required when this one is present I.e. when
    /// using this argument, the following arguments *must* be present.
    ///
    /// **NOTE:** [Conflicting] rules and [override] rules take precedence over being required
    /// by default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .requires_all(&["input", "output"])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::requires_all(&[arg, arg2])`] requires that all the arguments be used at
    /// runtime if the defining argument is used. If the defining argument isn't used, the other
    /// argument isn't required
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires("input")
    ///         .long("config"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .arg(Arg::with_name("output")
    ///         .index(2))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use cfg, so input and output weren't required
    /// ```
    ///
    /// Setting [`Arg::requires_all(&[arg, arg2])`] and *not* supplying all the arguments is an
    /// error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires_all(&["input", "output"])
    ///         .long("config"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .arg(Arg::with_name("output")
    ///         .index(2))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "file.conf", "in.txt"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// // We didn't use output
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [override]: ./struct.Arg.html#method.overrides_with
    /// [`Arg::requires_all(&[arg, arg2])`]: ./struct.Arg.html#method.requires_all
    pub fn requires_all(mut self, names: &[&'key str]) -> Self {
        if let Some(ref mut vec) = self.b.requires {
            for s in names {
                vec.push((None, s));
            }
        } else {
            let mut vec = vec![];
            for s in names {
                vec.push((None, *s));
            }
            self.b.requires = Some(vec);
        }
        self
    }

    /// Specifies the index of a positional argument **starting at** 1.
    ///
    /// **NOTE:** The index refers to position according to **other positional argument**. It does
    /// not define position in the argument list as a whole.
    ///
    /// **NOTE:** If no [`Arg::short`], or [`Arg::long`] have been defined, you can optionally
    /// leave off the `index` method, and the index will be assigned in order of evaluation.
    /// Utilizing the `index` method allows for setting indexes out of order
    ///
    /// **NOTE:** When utilized with [`Arg::multiple(true)`], only the **last** positional argument
    /// may be defined as multiple (i.e. with the highest index)
    ///
    /// # Panics
    ///
    /// Although not in this method directly, [`App`] will [`panic!`] if indexes are skipped (such
    /// as defining `index(1)` and `index(3)` but not `index(2)`, or a positional argument is
    /// defined as multiple and is not the highest index
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .index(1)
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .index(1))
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug"))
    ///     .get_matches_from(vec![
    ///         "prog", "--debug", "fast"
    ///     ]);
    ///
    /// assert!(m.is_present("mode"));
    /// assert_eq!(m.value_of("mode"), Some("fast")); // notice index(1) means "first positional"
    ///                                               // *not* first argument
    /// ```
    /// [`Arg::short`]: ./struct.Arg.html#method.short
    /// [`Arg::long`]: ./struct.Arg.html#method.long
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    /// [`App`]: ./struct.App.html
    /// [`panic!`]: https://doc.rust-lang.org/std/macro.panic!.html
    pub fn index(mut self, idx: u64) -> Self {
        self.index = Some(idx);
        self
    }

    /// Specifies a value that *stops* parsing multiple values of a give argument. By default when
    /// one sets [`multiple(true)`] on an argument, clap will continue parsing values for that
    /// argument until it reaches another valid argument, or one of the other more specific settings
    /// for multiple values is used (such as [`min_values`], [`max_values`] or
    /// [`number_of_values`]).
    ///
    /// **NOTE:** This setting only applies to [options] and [positional arguments]
    ///
    /// **NOTE:** When the terminator is passed in on the command line, it is **not** stored as one
    /// of the vaues
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("vals")
    ///     .takes_value(true)
    ///     .multiple(true)
    ///     .value_terminator(";")
    /// # ;
    /// ```
    /// The following example uses two arguments, a sequence of commands, and the location in which
    /// to perform them
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cmds")
    ///         .multiple(true)
    ///         .allow_hyphen_values(true)
    ///         .value_terminator(";"))
    ///     .arg(Arg::with_name("location"))
    ///     .get_matches_from(vec![
    ///         "prog", "find", "-type", "f", "-name", "special", ";", "/home/clap"
    ///     ]);
    /// let cmds: Vec<_> = m.values_of("cmds").unwrap().collect();
    /// assert_eq!(&cmds, &["find", "-type", "f", "-name", "special"]);
    /// assert_eq!(m.value_of("location"), Some("/home/clap"));
    /// ```
    /// [options]: ./struct.Arg.html#method.takes_value
    /// [positional arguments]: ./struct.Arg.html#method.index
    /// [`multiple(true)`]: ./struct.Arg.html#method.multiple
    /// [`min_values`]: ./struct.Arg.html#method.min_values
    /// [`number_of_values`]: ./struct.Arg.html#method.number_of_values
    /// [`max_values`]: ./struct.Arg.html#method.max_values
    pub fn value_terminator(mut self, term: &'other str) -> Self {
        self.setb(ArgSettings::TakesValue);
        self.v.terminator = Some(term);
        self
    }

    /// Specifies a list of possible values for this argument. At runtime, `clap` verifies that
    /// only one of the specified values was used, or fails with an error message.
    ///
    /// **NOTE:** This setting only applies to [options] and [positional arguments]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("mode")
    ///     .takes_value(true)
    ///     .possible_values(&["fast", "slow", "medium"])
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true)
    ///         .possible_values(&["fast", "slow", "medium"]))
    ///     .get_matches_from(vec![
    ///         "prog", "--mode", "fast"
    ///     ]);
    /// assert!(m.is_present("mode"));
    /// assert_eq!(m.value_of("mode"), Some("fast"));
    /// ```
    ///
    /// The next example shows a failed parse from using a value which wasn't defined as one of the
    /// possible values.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true)
    ///         .possible_values(&["fast", "slow", "medium"]))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--mode", "wrong"
    ///     ]);
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::InvalidValue);
    /// ```
    /// [options]: ./struct.Arg.html#method.takes_value
    /// [positional arguments]: ./struct.Arg.html#method.index
    pub fn possible_values(mut self, names: &[&'other str]) -> Self {
        if let Some(ref mut vec) = self.v.possible_vals {
            for s in names {
                vec.push(s);
            }
        } else {
            self.v.possible_vals = Some(names.iter().map(|s| *s).collect::<Vec<_>>());
        }
        self
    }

    /// Specifies a possible value for this argument, one at a time. At runtime, `clap` verifies
    /// that only one of the specified values was used, or fails with error message.
    ///
    /// **NOTE:** This setting only applies to [options] and [positional arguments]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("mode")
    ///     .takes_value(true)
    ///     .possible_value("fast")
    ///     .possible_value("slow")
    ///     .possible_value("medium")
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true)
    ///         .possible_value("fast")
    ///         .possible_value("slow")
    ///         .possible_value("medium"))
    ///     .get_matches_from(vec![
    ///         "prog", "--mode", "fast"
    ///     ]);
    /// assert!(m.is_present("mode"));
    /// assert_eq!(m.value_of("mode"), Some("fast"));
    /// ```
    ///
    /// The next example shows a failed parse from using a value which wasn't defined as one of the
    /// possible values.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true)
    ///         .possible_value("fast")
    ///         .possible_value("slow")
    ///         .possible_value("medium"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--mode", "wrong"
    ///     ]);
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::InvalidValue);
    /// ```
    /// [options]: ./struct.Arg.html#method.takes_value
    /// [positional arguments]: ./struct.Arg.html#method.index
    pub fn possible_value(mut self, name: &'other str) -> Self {
        if let Some(ref mut vec) = self.v.possible_vals {
            vec.push(name);
        } else {
            self.v.possible_vals = Some(vec![name]);
        }
        self
    }

    /// Specifies the name of the [`ArgGroup`] the argument belongs to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .long("debug")
    ///     .group("mode")
    /// # ;
    /// ```
    ///
    /// Multiple arguments can be a member of a single group and then the group checked as if it
    /// was one of said arguments.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug")
    ///         .group("mode"))
    ///     .arg(Arg::with_name("verbose")
    ///         .long("verbose")
    ///         .group("mode"))
    ///     .get_matches_from(vec![
    ///         "prog", "--debug"
    ///     ]);
    /// assert!(m.is_present("mode"));
    /// ```
    /// [`ArgGroup`]: ./struct.ArgGroup.html
    pub fn group(mut self, name: &'key str) -> Self {
        if let Some(ref mut vec) = self.b.groups {
            vec.push(name);
        } else {
            self.b.groups = Some(vec![name]);
        }
        self
    }

    /// Specifies the names of multiple [`ArgGroup`]'s the argument belongs to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .long("debug")
    ///     .groups(&["mode", "verbosity"])
    /// # ;
    /// ```
    ///
    /// Arguments can be members of multiple groups and then the group checked as if it
    /// was one of said arguments.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug")
    ///         .groups(&["mode", "verbosity"]))
    ///     .arg(Arg::with_name("verbose")
    ///         .long("verbose")
    ///         .groups(&["mode", "verbosity"]))
    ///     .get_matches_from(vec![
    ///         "prog", "--debug"
    ///     ]);
    /// assert!(m.is_present("mode"));
    /// assert!(m.is_present("verbosity"));
    /// ```
    /// [`ArgGroup`]: ./struct.ArgGroup.html
    pub fn groups(mut self, names: &[&'key str]) -> Self {
        if let Some(ref mut vec) = self.b.groups {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.groups = Some(names.into_iter().map(|s| *s).collect::<Vec<_>>());
        }
        self
    }

    /// Specifies how many values are required to satisfy this argument. For example, if you had a
    /// `-f <file>` argument where you wanted exactly 3 'files' you would set
    /// `.number_of_values(3)`, and this argument wouldn't be satisfied unless the user provided
    /// 3 and only 3 values.
    ///
    /// **NOTE:** Does *not* require [`Arg::multiple(true)`] to be set. Setting
    /// [`Arg::multiple(true)`] would allow `-f <file> <file> <file> -f <file> <file> <file>` where
    /// as *not* setting [`Arg::multiple(true)`] would only allow one occurrence of this argument.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("file")
    ///     .short("f")
    ///     .number_of_values(3)
    /// # ;
    /// ```
    ///
    /// Not supplying the correct number of values is an error
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .number_of_values(2)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::WrongNumberOfValues);
    /// ```
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn number_of_values(mut self, qty: u64) -> Self {
        self.setb(ArgSettings::TakesValue);
        self.v.num_vals = Some(qty);
        self
    }

    /// Allows one to perform a custom validation on the argument value. You provide a closure
    /// which accepts a [`String`] value, and return a [`Result`] where the [`Err(String)`] is a
    /// message displayed to the user.
    ///
    /// **NOTE:** The error message does *not* need to contain the `error:` portion, only the
    /// message as all errors will appear as
    /// `error: Invalid value for '<arg>': <YOUR MESSAGE>` where `<arg>` is replaced by the actual
    /// arg, and `<YOUR MESSAGE>` is the `String` you return as the error.
    ///
    /// **NOTE:** There is a small performance hit for using validators, as they are implemented
    /// with [`Rc`] pointers. And the value to be checked will be allocated an extra time in order
    /// to to be passed to the closure. This performance hit is extremely minimal in the grand
    /// scheme of things.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// fn has_at(v: String) -> Result<(), String> {
    ///     if v.contains("@") { return Ok(()); }
    ///     Err(String::from("The value did not contain the required @ sigil"))
    /// }
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .index(1)
    ///         .validator(has_at))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "some@file"
    ///     ]);
    /// assert!(res.is_ok());
    /// assert_eq!(res.unwrap().value_of("file"), Some("some@file"));
    /// ```
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`Err(String)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    /// [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
    pub fn validator<F>(mut self, f: F) -> Self
        where F: Fn(String) -> Result<(), String> + 'static
    {
        self.v.validator = Some(Rc::new(f));
        self
    }

    /// Works identically to Validator but is intended to be used with values that could 
    /// contain non UTF-8 formatted strings.
    ///
    /// # Examples
    ///
    #[cfg_attr(not(unix), doc=" ```ignore")]
    #[cfg_attr(    unix , doc=" ```rust")]
    /// # use clap::{App, Arg};
    /// # use std::ffi::{OsStr, OsString};
    /// # use std::os::unix::ffi::OsStrExt;
    /// fn has_ampersand(v: &OsStr) -> Result<(), OsString> {
    ///     if v.as_bytes().iter().any(|b| *b == b'&') { return Ok(()); }
    ///     Err(OsString::from("The value did not contain the required & sigil"))
    /// }
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .index(1)
    ///         .validator_os(has_ampersand))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "Fish & chips"
    ///     ]);
    /// assert!(res.is_ok());
    /// assert_eq!(res.unwrap().value_of("file"), Some("Fish & chips"));
    /// ```
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    /// [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`Err(String)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    /// [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
    pub fn validator_os<F>(mut self, f: F) -> Self
        where F: Fn(&OsStr) -> Result<(), String> + 'static
    {
        self.v.validator_os = Some(Rc::new(f));
        self
    }

    /// Specifies the *maximum* number of values are for this argument. For example, if you had a
    /// `-f <file>` argument where you wanted up to 3 'files' you would set `.max_values(3)`, and
    /// this argument would be satisfied if the user provided, 1, 2, or 3 values.
    ///
    /// **NOTE:** This does *not* implicitly set [`Arg::multiple(true)`]. This is because
    /// `-o val -o val` is multiple occurrences but a single value and `-o val1 val2` is a single
    /// occurence with multiple values. For positional arguments this **does** set
    /// [`Arg::multiple(true)`] because there is no way to determine the difference between multiple
    /// occurences and multiple values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("file")
    ///     .short("f")
    ///     .max_values(3)
    /// # ;
    /// ```
    ///
    /// Supplying less than the maximum number of values is allowed
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .max_values(3)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1", "file2"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// let m = res.unwrap();
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2"]);
    /// ```
    ///
    /// Supplying more than the maximum number of values is an error
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .max_values(2)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1", "file2", "file3"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::TooManyValues);
    /// ```
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn max_values(mut self, qty: u64) -> Self {
        self.setb(ArgSettings::TakesValue);
        self.v.max_vals = Some(qty);
        self
    }

    /// Specifies the *minimum* number of values for this argument. For example, if you had a
    /// `-f <file>` argument where you wanted at least 2 'files' you would set
    /// `.min_values(2)`, and this argument would be satisfied if the user provided, 2 or more
    /// values.
    ///
    /// **NOTE:** This does not implicitly set [`Arg::multiple(true)`]. This is because
    /// `-o val -o val` is multiple occurrences but a single value and `-o val1 val2` is a single
    /// occurence with multiple values. For positional arguments this **does** set
    /// [`Arg::multiple(true)`] because there is no way to determine the difference between multiple
    /// occurences and multiple values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("file")
    ///     .short("f")
    ///     .min_values(3)
    /// # ;
    /// ```
    ///
    /// Supplying more than the minimum number of values is allowed
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .min_values(2)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1", "file2", "file3"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// let m = res.unwrap();
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2", "file3"]);
    /// ```
    ///
    /// Supplying less than the minimum number of values is an error
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .min_values(2)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::TooFewValues);
    /// ```
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn min_values(mut self, qty: u64) -> Self {
        self.v.min_vals = Some(qty);
        self.set(ArgSettings::TakesValue)
    }

    /// Specifies the separator to use when values are clumped together, defaults to `,` (comma).
    ///
    /// **NOTE:** implicitly sets [`Arg::use_delimiter(true)`]
    ///
    /// **NOTE:** implicitly sets [`Arg::takes_value(true)`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("config")
    ///         .short("c")
    ///         .long("config")
    ///         .value_delimiter(";"))
    ///     .get_matches_from(vec![
    ///         "prog", "--config=val1;val2;val3"
    ///     ]);
    ///
    /// assert_eq!(m.values_of("config").unwrap().collect::<Vec<_>>(), ["val1", "val2", "val3"])
    /// ```
    /// [`Arg::use_delimiter(true)`]: ./struct.Arg.html#method.use_delimiter
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    pub fn value_delimiter(mut self, d: &str) -> Self {
        self.unsetb(ArgSettings::ValueDelimiterNotSet);
        self.setb(ArgSettings::TakesValue);
        self.setb(ArgSettings::UseValueDelimiter);
        self.v.val_delim = Some(d.chars()
            .nth(0)
            .expect("Failed to get value_delimiter from arg"));
        self
    }

    /// Specify multiple names for values of option arguments. These names are cosmetic only, used
    /// for help and usage strings only. The names are **not** used to access arguments. The values
    /// of the arguments are accessed in numeric order (i.e. if you specify two names `one` and
    /// `two` `one` will be the first matched value, `two` will be the second).
    ///
    /// This setting can be very helpful when describing the type of input the user should be
    /// using, such as `FILE`, `INTERFACE`, etc. Although not required, it's somewhat convention to
    /// use all capital letters for the value name.
    ///
    /// **Pro Tip:** It may help to use [`Arg::next_line_help(true)`] if there are long, or
    /// multiple value names in order to not throw off the help text alignment of all options.
    ///
    /// **NOTE:** This implicitly sets [`Arg::number_of_values`] if the number of value names is
    /// greater than one. I.e. be aware that the number of "names" you set for the values, will be
    /// the *exact* number of values required to satisfy this argument
    ///
    /// **NOTE:** implicitly sets [`Arg::takes_value(true)`]
    ///
    /// **NOTE:** Does *not* require or imply [`Arg::multiple(true)`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("speed")
    ///     .short("s")
    ///     .value_names(&["fast", "slow"])
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("io")
    ///         .long("io-files")
    ///         .value_names(&["INFILE", "OUTFILE"]))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    /// Running the above program produces the following output
    ///
    /// ```notrust
    /// valnames
    ///
    /// USAGE:
    ///    valnames [FLAGS] [OPTIONS]
    ///
    /// FLAGS:
    ///     -h, --help       Prints help information
    ///     -V, --version    Prints version information
    ///
    /// OPTIONS:
    ///     --io-files <INFILE> <OUTFILE>    Some help text
    /// ```
    /// [`Arg::next_line_help(true)`]: ./struct.Arg.html#method.next_line_help
    /// [`Arg::number_of_values`]: ./struct.Arg.html#method.number_of_values
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn value_names(mut self, names: &[&'other str]) -> Self {
        self.setb(ArgSettings::TakesValue);
        if self.is_set(ArgSettings::ValueDelimiterNotSet) {
            self.unsetb(ArgSettings::ValueDelimiterNotSet);
            self.setb(ArgSettings::UseValueDelimiter);
        }
        if let Some(ref mut vals) = self.v.val_names {
            let mut l = vals.len();
            for s in names {
                vals.insert(l, s);
                l += 1;
            }
        } else {
            let mut vm = VecMap::new();
            for (i, n) in names.iter().enumerate() {
                vm.insert(i, *n);
            }
            self.v.val_names = Some(vm);
        }
        self
    }

    /// Specifies the name for value of [option] or [positional] arguments inside of help
    /// documentation. This name is cosmetic only, the name is **not** used to access arguments.
    /// This setting can be very helpful when describing the type of input the user should be
    /// using, such as `FILE`, `INTERFACE`, etc. Although not required, it's somewhat convention to
    /// use all capital letters for the value name.
    ///
    /// **NOTE:** implicitly sets [`Arg::takes_value(true)`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("cfg")
    ///     .long("config")
    ///     .value_name("FILE")
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("config")
    ///         .long("config")
    ///         .value_name("FILE"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    /// Running the above program produces the following output
    ///
    /// ```notrust
    /// valnames
    ///
    /// USAGE:
    ///    valnames [FLAGS] [OPTIONS]
    ///
    /// FLAGS:
    ///     -h, --help       Prints help information
    ///     -V, --version    Prints version information
    ///
    /// OPTIONS:
    ///     --config <FILE>     Some help text
    /// ```
    /// [option]: ./struct.Arg.html#method.takes_value
    /// [positional]: ./struct.Arg.html#method.index
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    pub fn value_name(mut self, name: &'other str) -> Self {
        self.setb(ArgSettings::TakesValue);
        if let Some(ref mut vals) = self.v.val_names {
            let l = vals.len();
            vals.insert(l, name);
        } else {
            let mut vm = VecMap::new();
            vm.insert(0, name);
            self.v.val_names = Some(vm);
        }
        self
    }

    /// Specifies the value of the argument when *not* specified at runtime.
    ///
    /// **NOTE:** If the user *does not* use this argument at runtime, [`ArgMatches::occurrences_of`]
    /// will return `0` even though the [`ArgMatches::value_of`] will return the default specified.
    ///
    /// **NOTE:** If the user *does not* use this argument at runtime [`ArgMatches::is_present`] will
    /// still return `true`. If you wish to determine whether the argument was used at runtime or
    /// not, consider [`ArgMatches::occurrences_of`] which will return `0` if the argument was *not*
    /// used at runtmie.
    ///
    /// **NOTE:** This setting is perfectly compatible with [`Arg::default_value_if`] but slightly
    /// different. `Arg::default_value` *only* takes affect when the user has not provided this arg
    /// at runtime. `Arg::default_value_if` however only takes affect when the user has not provided
    /// a value at runtime **and** these other conditions are met as well. If you have set
    /// `Arg::default_value` and `Arg::default_value_if`, and the user **did not** provide a this
    /// arg at runtime, nor did were the conditions met for `Arg::default_value_if`, the
    /// `Arg::default_value` will be applied.
    ///
    /// **NOTE:** This implicitly sets [`Arg::takes_value(true)`].
    ///
    /// **NOTE:** This setting effectively disables `AppSettings::ArgRequiredElseHelp` if used in
    /// conjuction as it ensures that some argument will always be present.
    ///
    /// # Examples
    ///
    /// First we use the default value without providing any value at runtime.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .long("myopt")
    ///         .default_value("myval"))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("opt"), Some("myval"));
    /// assert!(m.is_present("opt"));
    /// assert_eq!(m.occurrences_of("opt"), 0);
    /// ```
    ///
    /// Next we provide a value at runtime to override the default.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .long("myopt")
    ///         .default_value("myval"))
    ///     .get_matches_from(vec![
    ///         "prog", "--myopt=non_default"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("opt"), Some("non_default"));
    /// assert!(m.is_present("opt"));
    /// assert_eq!(m.occurrences_of("opt"), 1);
    /// ```
    /// [`ArgMatches::occurrences_of`]: ./struct.ArgMatches.html#method.occurrences_of
    /// [`ArgMatches::value_of`]: ./struct.ArgMatches.html#method.value_of
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`ArgMatches::is_present`]: ./struct.ArgMatches.html#method.is_present
    /// [`Arg::default_value_if`]: ./struct.Arg.html#method.default_value_if
    pub fn default_value(mut self, val: &'key str) -> Self {
        self.setb(ArgSettings::TakesValue);
        self.v.default_val = Some(OsStr::from_bytes(val.as_bytes()));
        self
    }

    /// Specifies the value of the argument if `arg` has been used at runtime. If `val` is set to
    /// `None`, `arg` only needs to be present. If `val` is set to `"some-val"` then `arg` must be
    /// present at runtime **and** have the value `val`.
    ///
    /// **NOTE:** This setting is perfectly compatible with [`Arg::default_value`] but slightly
    /// different. `Arg::default_value` *only* takes affect when the user has not provided this arg
    /// at runtime. This setting however only takes affect when the user has not provided a value at
    /// runtime **and** these other conditions are met as well. If you have set `Arg::default_value`
    /// and `Arg::default_value_if`, and the user **did not** provide a this arg at runtime, nor did
    /// were the conditions met for `Arg::default_value_if`, the `Arg::default_value` will be
    /// applied.
    ///
    /// **NOTE:** This implicitly sets [`Arg::takes_value(true)`].
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows (`None` can be represented
    /// as `null` in YAML)
    ///
    /// ```yaml
    /// default_value_if:
    ///     - [arg, val, default]
    /// ```
    ///
    /// # Examples
    ///
    /// First we use the default value only if another arg is present at runtime.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_if("flag", None, "default"))
    ///     .get_matches_from(vec![
    ///         "prog", "--flag"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), Some("default"));
    /// ```
    ///
    /// Next we run the same test, but without providing `--flag`.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_if("flag", None, "default"))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), None);
    /// ```
    ///
    /// Now lets only use the default value if `--opt` contains the value `special`.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .takes_value(true)
    ///         .long("opt"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_if("opt", Some("special"), "default"))
    ///     .get_matches_from(vec![
    ///         "prog", "--opt", "special"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), Some("default"));
    /// ```
    ///
    /// We can run the same test and provide any value *other than* `special` and we won't get a
    /// default value.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .takes_value(true)
    ///         .long("opt"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_if("opt", Some("special"), "default"))
    ///     .get_matches_from(vec![
    ///         "prog", "--opt", "hahaha"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), None);
    /// ```
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg::default_value`]: ./struct.Arg.html#method.default_value
    pub fn default_value_if(self, arg: &'key str, val: Option<&'other str>, default: &'other str) -> Self {
        self.default_value_if_os(arg,
                                 val.map(str::as_bytes).map(OsStr::from_bytes),
                                 OsStr::from_bytes(default.as_bytes()))
    }

    /// Provides a conditional default value in the exact same manner as [`Arg::default_value_if`]
    /// only using [`OsStr`]s instead.
    /// [`Arg::default_value_if`]: ./struct.Arg.html#method.default_value_if
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    pub fn default_value_if_os(mut self,
                               arg: &'key str,
                               val: Option<&'other OsStr>,
                               default: &'other OsStr)
                               -> Self {
        self.setb(ArgSettings::TakesValue);
        if let Some(ref mut vm) = self.v.default_vals_ifs {
            let l = vm.len();
            vm.insert(l, (arg, val, default));
        } else {
            let mut vm = VecMap::new();
            vm.insert(0, (arg, val, default));
            self.v.default_vals_ifs = Some(vm);
        }
        self
    }

    /// Specifies multiple values and conditions in the same manner as [`Arg::default_value_if`].
    /// The method takes a slice of tuples in the `(arg, Option<val>, default)` format.
    ///
    /// **NOTE**: The conditions are stored in order and evaluated in the same order. I.e. the first
    /// if multiple conditions are true, the first one found will be applied and the ultimate value.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// default_value_if:
    ///     - [arg, val, default]
    ///     - [arg2, null, default2]
    /// ```
    ///
    /// # Examples
    ///
    /// First we use the default value only if another arg is present at runtime.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("opt")
    ///         .long("opt")
    ///         .takes_value(true))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_ifs(&[
    ///             ("flag", None, "default"),
    ///             ("opt", Some("channal"), "chan"),
    ///         ]))
    ///     .get_matches_from(vec![
    ///         "prog", "--opt", "channal"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), Some("chan"));
    /// ```
    ///
    /// Next we run the same test, but without providing `--flag`.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_ifs(&[
    ///             ("flag", None, "default"),
    ///             ("opt", Some("channal"), "chan"),
    ///         ]))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), None);
    /// ```
    ///
    /// We can also see that these values are applied in order, and if more than one condition is
    /// true, only the first evaluatd "wins"
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("opt")
    ///         .long("opt")
    ///         .takes_value(true))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_ifs(&[
    ///             ("flag", None, "default"),
    ///             ("opt", Some("channal"), "chan"),
    ///         ]))
    ///     .get_matches_from(vec![
    ///         "prog", "--opt", "channal", "--flag"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), Some("default"));
    /// ```
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg::default_value`]: ./struct.Arg.html#method.default_value
    pub fn default_value_ifs(mut self, ifs: &[(&'key str, Option<&'other str>, &'other str)]) -> Self {
        for &(arg, val, default) in ifs {
            self = self.default_value_if_os(arg,
                                            val.map(str::as_bytes).map(OsStr::from_bytes),
                                            OsStr::from_bytes(default.as_bytes()));
        }
        self
    }

    /// Provides multiple conditional default values in the exact same manner as
    /// [`Arg::default_value_ifs`] only using [`OsStr`]s instead.
    /// [`Arg::default_value_ifs`]: ./struct.Arg.html#method.default_value_ifs
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    #[cfg_attr(feature = "lints", allow(explicit_counter_loop))]
    pub fn default_value_ifs_os(mut self, ifs: &[(&'key str, Option<&'other OsStr>, &'other OsStr)]) -> Self {
        for &(arg, val, default) in ifs {
            self = self.default_value_if_os(arg, val, default);
        }
        self
    }

    /// Allows custom ordering of args within the help message. Args with a lower value will be
    /// displayed first in the help message. This is helpful when one would like to emphasise
    /// frequently used args, or prioritize those towards the top of the list. Duplicate values
    /// **are** allowed. Args with duplicate display orders will be displayed in alphabetical
    /// order.
    ///
    /// **NOTE:** The default is 999 for all arguments.
    ///
    /// **NOTE:** This setting is ignored for [positional arguments] which are always displayed in
    /// [index] order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("a") // Typically args are grouped alphabetically by name.
    ///                              // Args without a display_order have a value of 999 and are
    ///                              // displayed alphabetically with all other 999 valued args.
    ///         .long("long-option")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .help("Some help and text"))
    ///     .arg(Arg::with_name("b")
    ///         .long("other-option")
    ///         .short("O")
    ///         .takes_value(true)
    ///         .display_order(1)   // In order to force this arg to appear *first*
    ///                             // all we have to do is give it a value lower than 999.
    ///                             // Any other args with a value of 1 will be displayed
    ///                             // alphabetically with this one...then 2 values, then 3, etc.
    ///         .help("I should be first!"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays the following help message
    ///
    /// ```notrust
    /// cust-ord
    ///
    /// USAGE:
    ///     cust-ord [FLAGS] [OPTIONS]
    ///
    /// FLAGS:
    ///     -h, --help       Prints help information
    ///     -V, --version    Prints version information
    ///
    /// OPTIONS:
    ///     -O, --other-option <b>    I should be first!
    ///     -o, --long-option <a>     Some help and text
    /// ```
    /// [positional arguments]: ./struct.Arg.html#method.index
    /// [index]: ./struct.Arg.html#method.index
    pub fn display_order(mut self, ord: usize) -> Self {
        self.s.disp_ord = ord;
        self
    }

    /// Sets one of the [`ArgSettings`] settings for the argument
    /// [`ArgSettings`]: ./enum.ArgSettings.html
    pub fn setting(mut self, s: ArgSettings) -> Self {
        self.setb(s);
        self
    }

    /// Sets one of the [`ArgSettings`] settings for the argument
    /// [`ArgSettings`]: ./enum.ArgSettings.html
    pub fn settings(mut self, s: &[ArgSettings]) -> Self {
        for set in s {
            self.setb(set);
        }
        self
    }

    /// Unsets one of the [`ArgSettings`] settings for the argument
    /// [`ArgSettings`]: ./enum.ArgSettings.html
    pub fn unset_setting(mut self, s: ArgSettings) -> Self {
        self.unsetb(s);
        self
    }

    /// Sets one of the [`ArgSettings`] settings for the argument
    /// [`ArgSettings`]: ./enum.ArgSettings.html
    pub fn unset_settings(mut self, s: &[ArgSettings]) -> Self {
        for set in s {
            self.unsetb(set);
        }
        self
    }

    /// Checks if one of the [`ArgSettings`] settings is set for the argument
    /// [`ArgSettings`]: ./enum.ArgSettings.html
    pub fn is_set(&self, s: ArgSettings) -> bool { self.b.is_set(s) }

    #[doc(hidden)]
    pub fn setb(&mut self, s: ArgSettings) { self.b.set(s); }

    #[doc(hidden)]
    pub fn unsetb(&mut self, s: ArgSettings) { self.b.unset(s); }
}

impl<'key, 'other> PartialEq for Arg<'key, 'other> {
    fn eq(&self, other: &Arg<'key, 'other>) -> bool {
        self.b == other.b
    }
}
