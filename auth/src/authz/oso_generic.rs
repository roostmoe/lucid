use std::collections::BTreeSet;

use anyhow::{Context, ensure};
use oso::{Oso, PolarClass};
use tracing::info;

const LUCID_AUTHZ_CONFIG_BASE: &str = include_str!("lucid.polar");

pub(super) struct Init {
    pub polar_snippet: &'static str,
    pub polar_class: oso::Class,
}

pub struct OsoInit {
    pub oso: Oso,
    pub class_names: BTreeSet<String>,
}

pub struct OsoInitBuilder {
    oso: Oso,
    class_names: BTreeSet<String>,
    snippets: Vec<&'static str>,
}

impl OsoInitBuilder {
    pub fn new() -> OsoInitBuilder {
        OsoInitBuilder {
            oso: Oso::new(),
            class_names: BTreeSet::new(),
            snippets: vec![LUCID_AUTHZ_CONFIG_BASE],
        }
    }

    /// Registers a class that either has no associated polar snippet or whose
    /// snippet is part of the base file
    pub fn register_class(
        mut self,
        c: oso::Class,
    ) -> Result<Self, anyhow::Error> {
        info!(class = &c.name, "registering Oso class");
        let name = c.name.clone();
        let new_element = self.class_names.insert(name.clone());
        ensure!(new_element, "Oso class already registered: {:?}", &name);
        self.oso
            .register_class(c)
            .with_context(|| format!("registering Oso class {:?}", name));
        Ok(self)
    }

    /// Registers a class with associated Polar snippet
    pub(super) fn register_class_with_snippet(
        mut self,
        init: Init,
    ) -> Result<Self, anyhow::Error> {
        self.snippets.push(init.polar_snippet);
        self.register_class(init.polar_class)
    }

    pub fn build(mut self) -> Result<OsoInit, anyhow::Error> {
        let polar_config = self.snippets.join("\n");
        info!(config = &polar_config, "full Oso configuration");
        self.oso
            .load_str(&polar_config)
            .context("loading Polar (Oso) config")?;
        Ok(OsoInit { oso: self.oso, class_names: self.class_names })
    }
}

pub fn make_lucid_oso() -> Result<OsoInit, anyhow::Error> {
    let mut oso_builder = OsoInitBuilder::new();

    // Handwritten classes
    let classes = [
        Action::get_polar_class(),
    ];
    for c in classes {
        oso_builder = oso_builder.register_class(c)?;
    }

    // Macro-generated classes
    let generated_units = [];
    for init in generated_units {
        oso_builder = oso_builder.register_class_with_snippet(init)?;
    }

    oso_builder.build()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Query, // only used for `Database`
    List,
    Get,
    Create,
    Update,
    Delete,
    GetPolicy,
    SetPolicy,
}

impl oso::PolarClass for Action {
    fn get_polar_class_builder() -> oso::ClassBuilder<Self> {
        oso::Class::builder()
            .with_equality_check()
            .add_method("to_perm", |a: &Action| Perm::from(a).to_string())
    }
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Perm {
    Query, // only used for `Database`
    List,
    Get,
    Create,
    Update,
    Delete,
}

impl From<&Action> for Perm {
    fn from(a: &Action) -> Self {
        match a {
            Action::Query => Perm::Query,
            Action::Get => Perm::Get,
            Action::List => Perm::List,
            Action::Create => Perm::Create,
            Action::Update => Perm::Update,
            Action::Delete => Perm::Delete,
            Action::GetPolicy => Perm::Get,
            Action::SetPolicy => Perm::Update,
        }
    }
}

impl std::fmt::Display for Perm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Perm::Query => "query",
            Perm::List => "list",
            Perm::Get => "get",
            Perm::Create => "create",
            Perm::Update => "update",
            Perm::Delete => "delete",
        })
    }
}
