// Internal
use crate::util::{Id, Key};

#[cfg(feature = "yaml")]
use yaml_rust::Yaml;

/// `ArgGroup`s are a family of related [arguments] and way for you to express, "Any of these
/// arguments". By placing arguments in a logical group, you can create easier requirement and
/// exclusion rules instead of having to list each argument individually, or when you want a rule
/// to apply "any but not all" arguments.
///
/// For instance, you can make an entire `ArgGroup` required. If [`ArgGroup::multiple(true)`] is
/// set, this means that at least one argument from that group must be present. If
/// [`ArgGroup::multiple(false)`] is set (the default), one and *only* one must be present.
///
/// You can also do things such as name an entire `ArgGroup` as a [conflict] or [requirement] for
/// another argument, meaning any of the arguments that belong to that group will cause a failure
/// if present, or must present respectively.
///
/// Perhaps the most common use of `ArgGroup`s is to require one and *only* one argument to be
/// present out of a given set. Imagine that you had multiple arguments, and you want one of them
/// to be required, but making all of them required isn't feasible because perhaps they conflict
/// with each other. For example, lets say that you were building an application where one could
/// set a given version number by supplying a string with an option argument, i.e.
/// `--set-ver v1.2.3`, you also wanted to support automatically using a previous version number
/// and simply incrementing one of the three numbers. So you create three flags `--major`,
/// `--minor`, and `--patch`. All of these arguments shouldn't be used at one time but you want to
/// specify that *at least one* of them is used. For this, you can create a group.
///
/// Finally, you may use `ArgGroup`s to pull a value from a group of arguments when you don't care
/// exactly which argument was actually used at runtime.
///
/// # Examples
///
/// The following example demonstrates using an `ArgGroup` to ensure that one, and only one, of
/// the arguments from the specified group is present at runtime.
///
/// ```rust
/// # use clap::{App, ArgGroup, ErrorKind};
/// let result = App::new("app")
///     .arg("--set-ver [ver] 'set the version manually'")
///     .arg("--major         'auto increase major'")
///     .arg("--minor         'auto increase minor'")
///     .arg("--patch         'auto increase patch'")
///     .group(ArgGroup::new("vers")
///          .args(&["set-ver", "major", "minor", "patch"])
///          .required(true))
///     .try_get_matches_from(vec!["app", "--major", "--patch"]);
/// // Because we used two args in the group it's an error
/// assert!(result.is_err());
/// let err = result.unwrap_err();
/// assert_eq!(err.kind, ErrorKind::ArgumentConflict);
/// ```
/// This next example shows a passing parse of the same scenario
///
/// ```rust
/// # use clap::{App, ArgGroup};
/// let result = App::new("app")
///     .arg("--set-ver [ver] 'set the version manually'")
///     .arg("--major         'auto increase major'")
///     .arg("--minor         'auto increase minor'")
///     .arg("--patch         'auto increase patch'")
///     .group(ArgGroup::new("vers")
///          .args(&["set-ver", "major", "minor","patch"])
///          .required(true))
///     .try_get_matches_from(vec!["app", "--major"]);
/// assert!(result.is_ok());
/// let matches = result.unwrap();
/// // We may not know which of the args was used, so we can test for the group...
/// assert!(matches.is_present("vers"));
/// // we could also alternatively check each arg individually (not shown here)
/// ```
/// [`ArgGroup::multiple(true)`]: ./struct.ArgGroup.html#method.multiple
/// [arguments]: ./struct.Arg.html
/// [conflict]: ./struct.Arg.html#method.conflicts_with
/// [requirement]: ./struct.Arg.html#method.requires
#[derive(Default, Debug)]
pub struct ArgGroup<'help> {
    pub(crate) id: Id,
    pub(crate) name: &'help str,
    pub(crate) args: Vec<Id>,
    pub(crate) required: bool,
    pub(crate) requires: Vec<Id>,
    pub(crate) conflicts: Vec<Id>,
    pub(crate) multiple: bool,
}

impl<'help> ArgGroup<'help> {
    pub(crate) fn with_id(id: Id) -> Self {
        ArgGroup {
            id,
            ..ArgGroup::default()
        }
    }

    /// Creates a new instance of `ArgGroup` using a unique string name. The name will be used to
    /// get values from the group or refer to the group inside of conflict and requirement rules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, ArgGroup};
    /// ArgGroup::new("config")
    /// # ;
    /// ```
    pub fn new<S: Into<&'help str>>(n: S) -> Self {
        let name = n.into();
        ArgGroup {
            id: Id::from(&*name),
            name,
            ..ArgGroup::default()
        }
    }

    /// Adds an [argument] to this group by name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup};
    /// let m = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .arg("flag")
    ///         .arg("color"))
    ///     .get_matches_from(vec!["myprog", "-f"]);
    /// // maybe we don't know which of the two flags was used...
    /// assert!(m.is_present("req_flags"));
    /// // but we can also check individually if needed
    /// assert!(m.is_present("flag"));
    /// ```
    /// [argument]: ./struct.Arg.html
    pub fn arg<T: Key>(mut self, arg_id: T) -> Self {
        self.args.push(arg_id.into());
        self
    }

    /// Adds multiple [arguments] to this group by name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup};
    /// let m = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .args(&["flag", "color"]))
    ///     .get_matches_from(vec!["myprog", "-f"]);
    /// // maybe we don't know which of the two flags was used...
    /// assert!(m.is_present("req_flags"));
    /// // but we can also check individually if needed
    /// assert!(m.is_present("flag"));
    /// ```
    /// [arguments]: ./struct.Arg.html
    pub fn args<T: Key>(mut self, ns: &[T]) -> Self {
        for n in ns {
            self = self.arg(n);
        }
        self
    }

    /// Allows more than one of the [`Arg`]s in this group to be used. (Default: `false`)
    ///
    /// # Examples
    ///
    /// Notice in this example we use *both* the `-f` and `-c` flags which are both part of the
    /// group
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup};
    /// let m = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .args(&["flag", "color"])
    ///         .multiple(true))
    ///     .get_matches_from(vec!["myprog", "-f", "-c"]);
    /// // maybe we don't know which of the two flags was used...
    /// assert!(m.is_present("req_flags"));
    /// ```
    /// In this next example, we show the default behavior (i.e. `multiple(false)) which will throw
    /// an error if more than one of the args in the group was used.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .args(&["flag", "color"]))
    ///     .try_get_matches_from(vec!["myprog", "-f", "-c"]);
    /// // Because we used both args in the group it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::ArgumentConflict);
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    #[inline]
    pub fn multiple(mut self, m: bool) -> Self {
        self.multiple = m;
        self
    }

    /// Sets the group as required or not. A required group will be displayed in the usage string
    /// of the application in the format `<arg|arg2|arg3>`. A required `ArgGroup` simply states
    /// that one argument from this group *must* be present at runtime (unless
    /// conflicting with another argument).
    ///
    /// **NOTE:** This setting only applies to the current [`App`] / [``], and not
    /// globally.
    ///
    /// **NOTE:** By default, [`ArgGroup::multiple`] is set to `false` which when combined with
    /// `ArgGroup::required(true)` states, "One and *only one* arg must be used from this group.
    /// Use of more than one arg is an error." Vice setting `ArgGroup::multiple(true)` which
    /// states, '*At least* one arg from this group must be used. Using multiple is OK."
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .args(&["flag", "color"])
    ///         .required(true))
    ///     .try_get_matches_from(vec!["myprog"]);
    /// // Because we didn't use any of the args in the group, it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`App`]: ./struct.App.html
    /// [``]: ./struct..html
    /// [`ArgGroup::multiple`]: ./struct.ArgGroup.html#method.multiple
    #[inline]
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }

    /// Sets the requirement rules of this group. This is not to be confused with a
    /// [required group]. Requirement rules function just like [argument requirement rules], you
    /// can name other arguments or groups that must be present when any one of the arguments from
    /// this group is used.
    ///
    /// **NOTE:** The name provided may be an argument, or group name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .arg(Arg::new("debug")
    ///         .short('d'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .args(&["flag", "color"])
    ///         .requires("debug"))
    ///     .try_get_matches_from(vec!["myprog", "-c"]);
    /// // because we used an arg from the group, and the group requires "-d" to be used, it's an
    /// // error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [required group]: ./struct.ArgGroup.html#method.required
    /// [argument requirement rules]: ./struct.Arg.html#method.requires
    pub fn requires<T: Key>(mut self, id: T) -> Self {
        self.requires.push(id.into());
        self
    }

    /// Sets the requirement rules of this group. This is not to be confused with a
    /// [required group]. Requirement rules function just like [argument requirement rules], you
    /// can name other arguments or groups that must be present when one of the arguments from this
    /// group is used.
    ///
    /// **NOTE:** The names provided may be an argument, or group name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .arg(Arg::new("debug")
    ///         .short('d'))
    ///     .arg(Arg::new("verb")
    ///         .short('v'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .args(&["flag", "color"])
    ///         .requires_all(&["debug", "verb"]))
    ///     .try_get_matches_from(vec!["myprog", "-c", "-d"]);
    /// // because we used an arg from the group, and the group requires "-d" and "-v" to be used,
    /// // yet we only used "-d" it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [required group]: ./struct.ArgGroup.html#method.required
    /// [argument requirement rules]: ./struct.Arg.html#method.requires_all
    pub fn requires_all(mut self, ns: &[&'help str]) -> Self {
        for n in ns {
            self = self.requires(n);
        }
        self
    }

    /// Sets the exclusion rules of this group. Exclusion (aka conflict) rules function just like
    /// [argument exclusion rules], you can name other arguments or groups that must *not* be
    /// present when one of the arguments from this group are used.
    ///
    /// **NOTE:** The name provided may be an argument, or group name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .arg(Arg::new("debug")
    ///         .short('d'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .args(&["flag", "color"])
    ///         .conflicts_with("debug"))
    ///     .try_get_matches_from(vec!["myprog", "-c", "-d"]);
    /// // because we used an arg from the group, and the group conflicts with "-d", it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::ArgumentConflict);
    /// ```
    /// [argument exclusion rules]: ./struct.Arg.html#method.conflicts_with
    pub fn conflicts_with<T: Key>(mut self, id: T) -> Self {
        self.conflicts.push(id.into());
        self
    }

    /// Sets the exclusion rules of this group. Exclusion rules function just like
    /// [argument exclusion rules], you can name other arguments or groups that must *not* be
    /// present when one of the arguments from this group are used.
    ///
    /// **NOTE:** The names provided may be an argument, or group name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("color")
    ///         .short('c'))
    ///     .arg(Arg::new("debug")
    ///         .short('d'))
    ///     .arg(Arg::new("verb")
    ///         .short('v'))
    ///     .group(ArgGroup::new("req_flags")
    ///         .args(&["flag", "color"])
    ///         .conflicts_with_all(&["debug", "verb"]))
    ///     .try_get_matches_from(vec!["myprog", "-c", "-v"]);
    /// // because we used an arg from the group, and the group conflicts with either "-v" or "-d"
    /// // it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::ArgumentConflict);
    /// ```
    /// [argument exclusion rules]: ./struct.Arg.html#method.conflicts_with_all
    pub fn conflicts_with_all(mut self, ns: &[&'help str]) -> Self {
        for n in ns {
            self = self.conflicts_with(n);
        }
        self
    }
}

impl<'help> From<&'_ ArgGroup<'help>> for ArgGroup<'help> {
    fn from(g: &ArgGroup<'help>) -> Self {
        ArgGroup {
            id: g.id.clone(),
            name: g.name,
            required: g.required,
            args: g.args.clone(),
            requires: g.requires.clone(),
            conflicts: g.conflicts.clone(),
            multiple: g.multiple,
        }
    }
}

#[cfg(feature = "yaml")]
impl<'help> From<&'help Yaml> for ArgGroup<'help> {
    /// Creates a new instance of `ArgGroup` from a .yaml (YAML) file.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use clap::{ArgGroup, load_yaml};
    /// let yaml = load_yaml!("group.yaml");
    /// let ag = ArgGroup::from(yaml);
    /// ```
    fn from(y: &'help Yaml) -> Self {
        let b = y.as_hash().expect("ArgGroup::from::<Yaml> expects a table");
        // We WANT this to panic on error...so expect() is good.
        let mut a = ArgGroup::default();
        let group_settings = if b.len() == 1 {
            let name_yaml = b.keys().next().expect("failed to get name");
            let name_str = name_yaml
                .as_str()
                .expect("failed to convert arg YAML name to str");
            a.name = name_str;
            b.get(name_yaml)
                .expect("failed to get name_str")
                .as_hash()
                .expect("failed to convert to a hash")
        } else {
            b
        };

        for (k, v) in group_settings {
            a = match k.as_str().unwrap() {
                "required" => a.required(v.as_bool().unwrap()),
                "multiple" => a.multiple(v.as_bool().unwrap()),
                "args" => yaml_vec_or_str!(a, v, arg),
                "arg" => {
                    if let Some(ys) = v.as_str() {
                        a = a.arg(ys);
                    }
                    a
                }
                "requires" => yaml_vec_or_str!(a, v, requires),
                "conflicts_with" => yaml_vec_or_str!(a, v, conflicts_with),
                "name" => {
                    if let Some(ys) = v.as_str() {
                        a.name = ys;
                    }
                    a
                }
                s => panic!(
                    "Unknown ArgGroup setting '{}' in YAML file for \
                     ArgGroup '{}'",
                    s, a.name
                ),
            }
        }

        a
    }
}

#[cfg(test)]
mod test {
    use super::ArgGroup;
    #[cfg(feature = "yaml")]
    use yaml_rust::YamlLoader;

    #[test]
    fn groups() {
        let g = ArgGroup::new("test")
            .arg("a1")
            .arg("a4")
            .args(&["a2", "a3"])
            .required(true)
            .conflicts_with("c1")
            .conflicts_with_all(&["c2", "c3"])
            .conflicts_with("c4")
            .requires("r1")
            .requires_all(&["r2", "r3"])
            .requires("r4");

        let args = vec!["a1".into(), "a4".into(), "a2".into(), "a3".into()];
        let reqs = vec!["r1".into(), "r2".into(), "r3".into(), "r4".into()];
        let confs = vec!["c1".into(), "c2".into(), "c3".into(), "c4".into()];

        assert_eq!(g.args, args);
        assert_eq!(g.requires, reqs);
        assert_eq!(g.conflicts, confs);
    }

    #[test]
    fn test_from() {
        let g = ArgGroup::new("test")
            .arg("a1")
            .arg("a4")
            .args(&["a2", "a3"])
            .required(true)
            .conflicts_with("c1")
            .conflicts_with_all(&["c2", "c3"])
            .conflicts_with("c4")
            .requires("r1")
            .requires_all(&["r2", "r3"])
            .requires("r4");

        let args = vec!["a1".into(), "a4".into(), "a2".into(), "a3".into()];
        let reqs = vec!["r1".into(), "r2".into(), "r3".into(), "r4".into()];
        let confs = vec!["c1".into(), "c2".into(), "c3".into(), "c4".into()];

        let g2 = ArgGroup::from(&g);
        assert_eq!(g2.args, args);
        assert_eq!(g2.requires, reqs);
        assert_eq!(g2.conflicts, confs);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_yaml() {
        let g_yaml = "name: test
args:
- a1
- a4
- a2
- a3
conflicts_with:
- c1
- c2
- c3
- c4
requires:
- r1
- r2
- r3
- r4";
        let yaml = &YamlLoader::load_from_str(g_yaml).expect("failed to load YAML file")[0];
        let g = ArgGroup::from(yaml);
        let args = vec!["a1".into(), "a4".into(), "a2".into(), "a3".into()];
        let reqs = vec!["r1".into(), "r2".into(), "r3".into(), "r4".into()];
        let confs = vec!["c1".into(), "c2".into(), "c3".into(), "c4".into()];
        assert_eq!(g.args, args);
        assert_eq!(g.requires, reqs);
        assert_eq!(g.conflicts, confs);
    }

    // This test will *fail to compile* if ArgGroup is not Send + Sync
    #[test]
    fn arg_group_send_sync() {
        fn foo<T: Send + Sync>(_: T) {}
        foo(ArgGroup::new("test"))
    }
}

impl Clone for ArgGroup<'_> {
    fn clone(&self) -> Self {
        ArgGroup {
            id: self.id.clone(),
            name: self.name,
            required: self.required,
            args: self.args.clone(),
            requires: self.requires.clone(),
            conflicts: self.conflicts.clone(),
            multiple: self.multiple,
        }
    }
}
