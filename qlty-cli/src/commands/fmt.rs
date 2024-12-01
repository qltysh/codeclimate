use crate::ui::format::TextFormatter;
use crate::{Arguments, CommandError, CommandSuccess, Trigger};
use anyhow::Result;
use clap::Args;
use duct::cmd;
use qlty_check::{planner::Planner, CheckFilter, Executor, Processor, Settings};
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Fmt {
    /// Check all files, not just changed
    #[arg(short, long)]
    pub all: bool,

    /// Disable progress bar
    #[arg(long)]
    pub no_progress: bool,

    /// Exit successfully regardless of linter errors
    #[arg(long)]
    pub no_error: bool,

    /// Sample results from a number of files for each linter
    #[arg(long)]
    pub sample: Option<usize>,

    /// Maximum number of concurrent jobs
    #[arg(long)]
    pub jobs: Option<u32>,

    /// Filter by plugin or check
    #[arg(long)]
    filter: Option<String>,

    #[arg(value_enum, long, default_value = "manual")]
    trigger: Trigger,

    /// Print verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(long)]
    pub upstream: Option<String>,

    /// Format files in the Git index
    #[arg(long, conflicts_with = "index_file")]
    pub index: bool,

    /// Format files in the specified Git index file
    #[arg(long, conflicts_with = "index")]
    pub index_file: Option<PathBuf>,

    /// Files to analyze
    pub paths: Vec<PathBuf>,
}

impl Fmt {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        // if self.index {
        //     let repository = workspace.repo()?;
        //     let index = repository.index()?;

        //     let head_commit = repository.head()?.peel_to_commit()?;
        //     let head_tree = head_commit.tree()?;
        //     let mut diff_opts = git2::DiffOptions::new();

        //     let diff = repository.diff_tree_to_index(
        //         Some(&head_tree),
        //         Some(&index),
        //         Some(&mut diff_opts),
        //     )?;

        //     for delta in diff.deltas() {
        //         if let Some(path) = delta.new_file().path() {
        //             dbg!(path);
        //         }
        //     }

        //     return CommandSuccess::ok();
        // }

        // if let Some(index_file) = &self.index_file {
        //     let repository = workspace.repo()?;
        //     let index = git2::Index::open(index_file)?;

        //     let head_commit = repository.head()?.peel_to_commit()?;
        //     let head_tree = head_commit.tree()?;
        //     let mut diff_opts = git2::DiffOptions::new();

        //     let diff = repository.diff_tree_to_index(
        //         Some(&head_tree),
        //         Some(&index),
        //         Some(&mut diff_opts),
        //     )?;

        //     for delta in diff.deltas() {
        //         if let Some(path) = delta.new_file().path() {
        //             dbg!(path);
        //         }
        //     }

        //     return CommandSuccess::ok();
        // }

        let settings = self.build_settings()?;
        let plan = Planner::new(ExecutionVerb::Fmt, &settings)?.compute()?;
        // dbg!(plan);
        // return CommandSuccess::ok();
        let executor = Executor::new(&plan);
        let results = executor.install_and_invoke()?;

        let mut processor = Processor::new(&plan, results);
        let report = processor.compute()?;

        if self.index || self.index_file.is_some() {
            self.git_add(&report.formatted)?;
        }

        let formatter = TextFormatter::new(&report, settings.verbose);
        formatter.write_to(&mut std::io::stdout())?;

        if !self.no_error && report.has_errors() {
            Err(CommandError::Lint)
        } else {
            Ok(CommandSuccess {
                trigger: Some(self.trigger),
                ..Default::default()
            })
        }
    }

    fn git_add(&self, paths: &[PathBuf]) -> Result<()> {
        let mut args = vec!["add"];

        for path in paths {
            if let Some(path_str) = path.to_str() {
                args.push(path_str);
            }
        }

        if args.len() > 1 {
            cmd("git", &args).run()?;
        }

        Ok(())
    }

    fn build_settings(&self) -> Result<Settings> {
        let mut settings = Settings::default();
        settings.root = Workspace::assert_within_git_directory()?;
        settings.verbose = self.verbose as usize;
        settings.sample = self.sample;
        settings.all = (self.sample.unwrap_or(0) > 0) || self.all;
        settings.jobs = self.jobs;
        settings.progress = !self.no_progress;
        settings.filters = CheckFilter::from_optional_list(self.filter.clone());
        settings.upstream = self.upstream.clone();
        settings.index = self.index;
        settings.index_file = self.index_file.clone();
        settings.paths = self.paths.clone();
        settings.trigger = self.trigger.into();

        Ok(settings)
    }
}
