use std::path::Path;

use crate::jvm::maven_module::MavenModuleAnalyzer;
use crate::psa_project::Project;
use crate::{files, Module, ProjectStructureAnalyzer};

pub trait ModuleAnalyzer {
    fn analysis(&self, project_path: &str, module_path: &str) -> Option<Module>;
    fn is_related(&self, project: &Project) -> bool;
}

pub struct JvmProjectStructureAnalyzer {
    module_analyzers: Vec<Box<dyn ModuleAnalyzer>>,
}

impl JvmProjectStructureAnalyzer {
    fn analysis_modules(&self, project: &Project) -> Vec<Module> {
        let mut modules = Vec::new();
        let module = self.analysis_module(project);
        match module {
            Some(module) => modules.push(module),
            _ => (),
        }
        modules
    }

    fn analysis_module(&self, project: &Project) -> Option<Module> {
        for module_analyzer in self.module_analyzers.iter() {
            return match module_analyzer.is_related(project) {
                true => module_analyzer.analysis(&project.absolute_path, &project.absolute_path),
                _ => continue,
            };
        }
        None
    }
}

impl Default for JvmProjectStructureAnalyzer {
    fn default() -> Self {
        JvmProjectStructureAnalyzer {
            module_analyzers: vec![Box::new(MavenModuleAnalyzer {})],
        }
    }
}

impl ProjectStructureAnalyzer for JvmProjectStructureAnalyzer {
    fn analysis(&self, project_path: &str) -> Project {
        let project_name = get_project_name(project_path);
        let build_file = get_build_file(project_path).unwrap();
        let project_type = get_project_type(build_file);

        let mut project = Project::new(project_name.as_str(), project_path, project_type.as_str());
        let modules = &mut self.analysis_modules(&project);
        project.add_modules(modules);
        project
    }

    fn is_related(&self, project_path: &str) -> bool {
        let files = files::list_file_names(project_path);
        for file_name in files.iter() {
            if is_build_file(file_name) {
                return true;
            }
        }
        false
    }
}

fn get_project_type(build_file: String) -> String {
    return match build_file.as_str() {
        "pom.xml" => "maven".to_string(),
        _ => "UnKnow".to_string(),
    };
}

fn get_build_file(path: &str) -> Option<String> {
    let files = files::list_file_names(Path::new(path));
    files.into_iter().find(|file| is_build_file(file))
}

fn get_project_name(project_path: &str) -> String {
    Path::new(project_path)
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
}

fn is_build_file(file_name: &str) -> bool {
    match file_name {
        "pom.xml" => true,
        "build.gradle" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::files::join_path;
    use crate::jvm::psa_jvm::JvmProjectStructureAnalyzer;
    use crate::{Project, ProjectStructureAnalyzer};

    #[test]
    fn should_analysis_maven_project_sub_modules() {
        let project = do_analysis(vec![
            "_fixtures",
            "projects",
            "java",
            "multi_mod_maven_project",
        ]);

        let modules = project.modules;
        let project_module = modules.get(0).unwrap();
        let module1 = project_module.sub_modules.get(0).unwrap();
        let module2 = project_module.sub_modules.get(1).unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(project_module.sub_modules.len(), 2);
        assert_eq!(project.project_type, "maven");
        assert_eq!(project_module.name, "multi_mod_maven_project");
        assert_eq!(module1.name, "module1");
        assert_eq!(module2.name, "module2");
    }

    #[test]
    fn should_detect_project_module_content_root() {
        let project = do_analysis(vec![
            "_fixtures",
            "projects",
            "java",
            "multi_mod_maven_project",
        ]);
        let modules = project.modules;
        let project_module = modules.get(0).unwrap();
        let project_content_root = &project_module.content_root;

        let expect_source_path = join_path("", vec!["src", "main", "java"]);
        assert_eq!(project_content_root.source_root.len(), 1);
        assert_eq!(
            project_content_root.source_root.get(0).unwrap().as_str(),
            expect_source_path.as_str()
        );

        let expect_resource_path = join_path("", vec!["src", "main", "resources"]);
        assert_eq!(project_content_root.resource_root.len(), 1);
        assert_eq!(
            project_content_root.resource_root.get(0).unwrap().as_str(),
            expect_resource_path.as_str()
        );

        let expect_test_source_root = join_path("", vec!["src", "test", "java"]);
        assert_eq!(project_content_root.test_source_root.len(), 1);
        assert_eq!(
            project_content_root
                .test_source_root
                .get(0)
                .unwrap()
                .as_str(),
            expect_test_source_root.as_str()
        );

        let expect_test_resources_root = join_path("", vec!["src", "test", "resources"]);
        assert_eq!(project_content_root.test_resource_root.len(), 1);
        assert_eq!(
            project_content_root.test_resource_root.get(0).unwrap(),
            expect_test_resources_root.as_str()
        );
    }

    #[test]
    fn should_detect_sub_module_content_root() {
        let project = do_analysis(vec![
            "_fixtures",
            "projects",
            "java",
            "multi_mod_maven_project",
        ]);

        let modules = project.modules;
        let project_module = modules.get(0).unwrap();
        let module1 = project_module.sub_modules.get(0).unwrap();
        let content_root = &module1.content_root;

        let expect_source_path = join_path("", vec!["src", "main", "java"]);
        assert_eq!(
            content_root.source_root.get(0).unwrap().as_str(),
            expect_source_path
        );

        let expect_test_source_root = join_path("", vec!["src", "test", "java"]);
        assert_eq!(
            content_root.test_source_root.get(0).unwrap().as_str(),
            expect_test_source_root.as_str()
        );

        let expect_test_source_root = join_path("", vec!["src", "test", "java"]);
        assert_eq!(
            content_root.test_source_root.get(0).unwrap().as_str(),
            expect_test_source_root.as_str()
        );

        let expect_test_resources_root = join_path("", vec!["src", "test", "resources"]);
        assert_eq!(
            content_root.test_resource_root.get(0).unwrap(),
            expect_test_resources_root.as_str()
        );
    }

    fn do_analysis(path: Vec<&str>) -> Project {
        let mut project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();

        for path in path.into_iter() {
            project_dir.push(path);
        }

        let analyzer = JvmProjectStructureAnalyzer::default();
        analyzer.analysis(project_dir.display().to_string().as_str())
    }
}
