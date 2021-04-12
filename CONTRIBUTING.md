# Contributing to rust-bitcoin

:+1::tada: First off, thanks for taking the time to contribute! :tada::+1:

The following is a set of guidelines for contributing to Rust Bitcoin 
implementation and other Rust Bitcoin-related projects, which are hosted in the 
[Rust Bitcoin Community](https://github.com/rust-bitcoin) on GitHub. These are 
mostly guidelines, not rules. Use your best judgment, and feel free to propose 
changes to this document in a pull request.

#### Table Of Contents

- [General](#general)
- [Communication channels](#communication-channels)
- [Asking questions](#asking-questions)
- [Contribution workflow](#contribution-workflow)
- [Branches information](#branches-information)
- [Peer review](#peer-review)
- [Coding conventions](#coding-conventions)
  * [Formatting](#formatting)
  * [Derivation](#derivation)
  * [MSRV](#msrv)
  * [Naming conventions](#naming-conventions)
- [Security](#security)
- [Testing](#testing)
- [Going further](#going-further)


## General

The Rust Bitcoin project operates an open contributor model where anyone is 
welcome to contribute towards development in the form of peer review, 
documentation, testing and patches.

Anyone is invited to contribute without regard to technical experience,
"expertise", OSS experience, age, or other concern. However, the development of
standards & reference implementations demands a high-level of rigor, adversarial
thinking, thorough testing and risk-minimization. Any bug may cost users real
money. That being said, we deeply welcome people contributing for the first time
to an open source project or pick up Rust while contributing. Don't be shy,
you'll learn.


## Communication channels

Communication about Rust Bitcoin happens primarily in #rust-bitcoin IRC chat on 
Freenode with the logs available at <http://gnusha.org/rust-bitcoin/>

Discussion about code base improvements happens in GitHub issues and on pull
requests.

Major projects are tracked [here](https://github.com/orgs/rust-bitcoin/projects).
Major milestones are tracked [here](https://github.com/rust-bitcoin/rust-bitcoin/milestones).


## Asking questions

> **Note:** Please don't file an issue to ask a question. You'll get faster 
> results by using the resources below.

We have a dedicated developer channel on IRC, #rust-bitcoin@freenode.net where 
you may get helpful advice if you have questions.


## Contribution workflow

The codebase is maintained using the "contributor workflow" where everyone
without exception contributes patch proposals using "pull requests". This
facilitates social contribution, easy testing and peer review.

To contribute a patch, the workflow is a as follows:

1. Fork Repository
2. Create topic branch
3. Commit patches

In general commits should be atomic and diffs should be easy to read. For this
reason do not mix any formatting fixes or code moves with actual code changes.
Further, each commit, individually, should compile and pass tests, in order to
ensure git bisect and other automated tools function properly.

When adding a new feature thought must be given to the long term technical debt.
Every new features should be covered by unit tests.

When refactoring, structure your PR to make it easy to review and don't hesitate
to split it into multiple small, focused PRs.

Commits should cover both the issue fixed and the solution's rationale.
These [guidelines](https://chris.beams.io/posts/git-commit/) should be kept in
mind.

To facilitate communication with other contributors, the project is making use
of GitHub's "assignee" field. First check that no one is assigned and then
comment suggesting that you're working on it. If someone is already assigned,
don't hesitate to ask if the assigned party or previous commenters are still
working on it if it has been awhile.


## Branches information

The main library development happens in the `master` branch. This branch must
always compile without errors (using GitHub CI). All external contributions are
made within PRs into this branch.

Each commitment within a PR to the `master` must
* compile without errors;
* contain all necessary tests for the introduced functional;
* contain all docs.


## Peer review

Anyone may participate in peer review which is expressed by comments in the pull
request. Typically reviewers will review the code for obvious errors, as well as
test out the patch set and opine on the technical merits of the patch. PR should
be reviewed first on the conceptual level before focusing on code style or
grammar fixes.


## Coding conventions

### Formatting

We plan to utilize `rustfmt` for keeping the code formatting consistent. 
However, this will be a gradual process since a thorough peer review is required
to make sure that no unintended changes are introduced with the change of 
formatting. Thus, all PRs introducing large blocks of re-formatted code will not
be reviewed.

The current plan is to phase it in over multiple commits or even multiple PRs,
which will introduce no other changes that re-formatting, such that each change 
may be independently re-reproduced by each reviewer. The first commit should add
attributes to disable fmt for some parts of the code and a second one does the 
formatting – so only the first one needs review, the rest will be reproducible.

You may check the [discussion on the formatting](https://github.com/rust-bitcoin/rust-bitcoin/issues/172)
and [how it is planned to coordinate it with crate refactoring](https://github.com/rust-bitcoin/rust-bitcoin/pull/525)

<!--
Rust-fmt should be used as a coding style recommendations in general, with a
default coding style. By default, Rustfmt uses a style which conforms to the
[Rust style guide][style guide] that has been formalized through the [style RFC
process][fmt rfcs]. It is also required to run `cargo fmt` to make the code
formatted according to `rustfmt` parameters 
-->

### Derivation

Derivations applied to a data structures should be standardized:

1. All non-error types should opportunistically derive, where it is possible,
   the following traits:
   - `Copy`
   - `Clone`
   - `PartialEq` and `Eq`
   - `PartialOrd` and `Ord`
   - `Hash`
   - `Debug`
   
   By "where possible" we mean that by default a code line
   ```rust
   #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
   ```
   must be placed before each struct, and then those of these traits, which can't
   be auto-derived because of the member field restrictions should be removed.

2. `Default` derivation should be performed whenever there is a rationale to 
   have default constructor initializing "empty" data structure, i.e. this 
   empty structure has a real use in the business logic *outside of the scope
   of testing or creating dumb data*. For instance, if the structure consists
   only of collection types which may be empty it should derive `Default` trait.

3. **Error types** (both structs and enums) must implement `Display` and `Error`
   traits manually, and should provide `Error::source` function if some of the
   error cases contain other error type.

4. `Display` should be implemented for all data types which may be presented to
   the end user (not developers!), for instance in command line or as a part of
   GUI. Here are some guidelines: 
   - Normally, `Display` implementation should not just repeat `Debug` and
     structure the data in some visually-acceptable way. 
   - One should pay attention to the ability of providing alternative ways of 
     data formatting with `{:#}` formatting string option, detectable by 
     `std::fmt::Formatter::alternate()` function. Other important options to
     look at are `align`, `fill`, `pad`, `precision` and `width`. 
   - When displaying the member fields it is important to consider the ability
     to pass them display formatting options; thus, 
     `Display::fmt(&self.field, f)?;` is preferable over 
     `write!(f, "{}", self.field)?;`

5. Serde serializers should be implemented for all data types which may persist
   or may be presented in the UI or API as JSON/YAML and other kinds of data 
   representations (in fact, these are all data types).

The discussion about trait derivation can be read at 
[the tracking issue](https://github.com/rust-bitcoin/rust-bitcoin/issues/555).

### MSRV

The Minimal Supported Rust Version (MSRV) is 0.29; it is enforced by our CI. 
Later we plan to increase MSRV to support Rust 2018 and you are welcome to check
the [tracking issue](https://github.com/rust-bitcoin/rust-bitcoin/issues/510).

### Naming conventions

Naming of data structures/enums and their fields/variants must follow names used
in Bitcoin Core, with except to:
- case, which should follow Rust standards (i.e. PascalCase for types and 
  snake_case for fields and variants)
- `C`-prefix, which should be omitted


## Security

Security is the primary focus of Rust-LNPBP; disclosure of security
vulnerabilities helps prevent user loss of funds. If you believe a vulnerability
may affect other  implementations, please inform them.

Note that Rust-LNPBP is currently considered "pre-production" during this time,
there is no special handling of security issues. Please simply open an issue on
Github.


## Testing

Related to the security aspect, rust bitcoin developers must be taking testing
very seriously. Due to the modular nature of the project, writing new functional
tests is easy and good test coverage of the codebase is an important goal.
Refactoring the project to enable fine-grained unit testing is also an ongoing
effort.

Fuzzing is heavily encouraged: feel free to add related material under `fuzz/`

Mutation testing is planned; any contribution there would be warmly welcomed.


## Going further

You may be interested by Jon Atack guide on 
[How to review Bitcoin Core PRs](https://github.com/jonatack/bitcoin-development/blob/master/how-to-review-bitcoin-core-prs.md)
and [How to make Bitcoin Core PRs](https://github.com/jonatack/bitcoin-development/blob/master/how-to-make-bitcoin-core-prs.md).
While there are differences between the projects in terms of context and 
maturity, many of the suggestions offered apply to this project.

Overall, have fun :)
