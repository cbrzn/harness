use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::constants::Implementation;
use crate::error::{MergeManifestError};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Workflow {
    pub format: Option<String>,
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation: Option<String>,
    jobs: serde_json::Value,
}

pub type ImportAbis = Vec<ImportAbi>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ImportAbi {
    uri: String,
    abi: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Project {
    name: Option<String>,
    #[serde(rename = "type")]
    _type: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Source {
    schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    import_abis: Option<ImportAbis>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Manifest {
    pub format: Option<String>,
    pub project: Option<Project>,
    pub source: Option<Source>,
}

impl Manifest {
    pub fn merge(self, custom: Manifest, implementation: Option<&str>) -> Result<Manifest, MergeManifestError> {
        let default_project = self.project.ok_or(MergeManifestError::ProjectNotFound)?;

        let project  = match custom.project {
            Some(p) => Some(Project {
                name: p.name.or(default_project.name),
                _type: p._type.or(default_project._type),
            }),
            _ => Some(default_project)
        };


        let default_source = self.source.ok_or(MergeManifestError::SourceNotFound)?;

        let source = match custom.source {
            Some(s) => {
                let mut import_abis = default_source.import_abis;
                if s.import_abis.is_some() && implementation.is_some() {
                    let imports = s.import_abis.unwrap().iter().map(|import_abi| {
                        let abi_path = Path::new(&import_abi.abi);
                        if !abi_path.ends_with("build/wrap.info") {
                            let abi = abi_path.join("implementations").join(implementation.unwrap()).join("build/wrap.info");
                            return ImportAbi {
                                uri: import_abi.clone().uri,
                                abi: abi.clone().to_str().unwrap().to_string()
                            }
                        }

                        return import_abi.clone()
                    }).collect::<ImportAbis>();
                    import_abis = Some(imports);
                }

                let source = Source {
                    schema: s.schema.or(default_source.schema),
                    module: s.module.or(default_source.module),
                    import_abis
                };
                Some(source)
            },
            _ => Some(default_source)
        };

        Ok(Self {
            format: self.format,
            project,
            source
        })
    }

    pub fn default(
        feature: &str,
        implementation: &Option<&Implementation<'_>>,
    ) -> Manifest {
        let (module, schema, _type) = match implementation {
            Some(i) => {
                (
                    Some(i.module.to_string()),
                    Some("../../schema.graphql".to_string()),
                    Some(format!("wasm/{}", i.name.to_string()))
                )
            },
            None => (None, Some("./schema.graphql".to_string()), Some("interface".to_string()))
        };
        Manifest {
            format: Some("0.2.0".to_string()),
            project: Some(Project {
                name: Some(feature.to_string()),
                _type,
            }),
            source: Some(Source {
                schema,
                module,
                import_abis: None,
            })
        }
    }
}