use std::collections::HashSet;
use serde_json::Value;
use log::debug;

/// Analyzes package.json to detect which devDependencies are needed for building
pub fn detect_build_dependencies(package_json: &Value) -> HashSet<String> {
    let mut required = HashSet::new();
    
    // Get scripts section
    if let Some(scripts) = package_json.get("scripts").and_then(|s| s.as_object()) {
        // Analyze build script
        if let Some(build_script) = scripts.get("build").and_then(|s| s.as_str()) {
            analyze_script_dependencies(build_script, &mut required);
        }
        
        // Analyze prebuild script
        if let Some(prebuild_script) = scripts.get("prebuild").and_then(|s| s.as_str()) {
            analyze_script_dependencies(prebuild_script, &mut required);
        }
        
        // Analyze postbuild script  
        if let Some(postbuild_script) = scripts.get("postbuild").and_then(|s| s.as_str()) {
            analyze_script_dependencies(postbuild_script, &mut required);
        }
        
        // Analyze start script if it includes build
        if let Some(start_script) = scripts.get("start").and_then(|s| s.as_str()) {
            if start_script.contains("build") {
                if let Some(build_script) = scripts.get("build").and_then(|s| s.as_str()) {
                    analyze_script_dependencies(build_script, &mut required);
                }
            }
        }
    }
    
    // Check for TypeScript configuration
    if package_json.get("devDependencies")
        .and_then(|deps| deps.get("typescript"))
        .is_some() {
        // If there's a tsconfig.json reference or typescript in devDeps, 
        // we likely need it
        required.insert("typescript".to_string());
    }
    
    debug!("Auto-detected build dependencies: {:?}", required);
    required
}

/// Analyzes a script command to determine what tools it uses
fn analyze_script_dependencies(script: &str, required: &mut HashSet<String>) {
    // TypeScript
    if script.contains("tsc") || script.contains("typescript") {
        required.insert("typescript".to_string());
    }
    
    // Babel
    if script.contains("babel") {
        required.insert("@babel/core".to_string());
        required.insert("@babel/cli".to_string());
        // Check for presets
        if script.contains("preset-env") {
            required.insert("@babel/preset-env".to_string());
        }
        if script.contains("preset-react") {
            required.insert("@babel/preset-react".to_string());
        }
        if script.contains("preset-typescript") {
            required.insert("@babel/preset-typescript".to_string());
        }
    }
    
    // Webpack
    if script.contains("webpack") {
        required.insert("webpack".to_string());
        required.insert("webpack-cli".to_string());
    }
    
    // Rollup
    if script.contains("rollup") {
        required.insert("rollup".to_string());
    }
    
    // Parcel
    if script.contains("parcel") {
        required.insert("parcel".to_string());
    }
    
    // Vite
    if script.contains("vite") {
        required.insert("vite".to_string());
    }
    
    // ESBuild
    if script.contains("esbuild") {
        required.insert("esbuild".to_string());
    }
    
    // SWC
    if script.contains("swc") {
        required.insert("@swc/core".to_string());
        required.insert("@swc/cli".to_string());
    }
    
    // Build tools
    if script.contains("gulp") {
        required.insert("gulp".to_string());
    }
    if script.contains("grunt") {
        required.insert("grunt".to_string());
    }
    
    // CSS preprocessors
    if script.contains("sass") || script.contains("scss") {
        required.insert("sass".to_string());
    }
    if script.contains("less") {
        required.insert("less".to_string());
    }
    if script.contains("stylus") {
        required.insert("stylus".to_string());
    }
    if script.contains("postcss") {
        required.insert("postcss".to_string());
        required.insert("postcss-cli".to_string());
    }
    
    // Other common build tools
    if script.contains("rimraf") {
        required.insert("rimraf".to_string());
    }
    if script.contains("copyfiles") {
        required.insert("copyfiles".to_string());
    }
    if script.contains("cross-env") {
        required.insert("cross-env".to_string());
    }
    if script.contains("concurrently") {
        required.insert("concurrently".to_string());
    }
    if script.contains("ts-node") {
        required.insert("ts-node".to_string());
        required.insert("typescript".to_string()); // ts-node needs typescript
    }
    if script.contains("tsx") {
        required.insert("tsx".to_string());
    }
    if script.contains("tsup") {
        required.insert("tsup".to_string());
        required.insert("typescript".to_string());
    }
}

/// Categories of devDependencies that are typically safe to skip
pub fn is_safe_to_skip(dep_name: &str) -> bool {
    // Testing frameworks
    if dep_name.starts_with("jest") || 
       dep_name.starts_with("mocha") ||
       dep_name.starts_with("chai") ||
       dep_name.starts_with("@testing-library/") ||
       dep_name.starts_with("sinon") ||
       dep_name.starts_with("supertest") ||
       dep_name.starts_with("cypress") ||
       dep_name.starts_with("playwright") ||
       dep_name.starts_with("puppeteer") ||
       dep_name == "vitest" ||
       dep_name == "ava" ||
       dep_name == "tap" ||
       dep_name == "tape" {
        return true;
    }
    
    // Linting and formatting
    if dep_name.starts_with("eslint") ||
       dep_name.starts_with("prettier") ||
       dep_name.starts_with("tslint") ||
       dep_name.starts_with("@typescript-eslint/") ||
       dep_name.starts_with("stylelint") ||
       dep_name == "standard" ||
       dep_name == "xo" {
        return true;
    }
    
    // Documentation
    if dep_name.starts_with("jsdoc") ||
       dep_name.starts_with("typedoc") ||
       dep_name.starts_with("@compodoc/") ||
       dep_name == "documentation" ||
       dep_name.starts_with("docusaurus") {
        return true;
    }
    
    // Git hooks
    if dep_name == "husky" ||
       dep_name == "lint-staged" ||
       dep_name == "pre-commit" {
        return true;
    }
    
    // Coverage tools
    if dep_name == "nyc" ||
       dep_name == "c8" ||
       dep_name.starts_with("coveralls") ||
       dep_name.starts_with("codecov") {
        return true;
    }
    
    false
}

/// Generate npm install command with specific dependencies
pub fn generate_selective_install_command(
    package_manager: &str,
    required_deps: &HashSet<String>,
    include_deps: &[String],
    skip_deps: &[String],
) -> String {
    // For now, if we need any devDependencies, install all
    // A more sophisticated approach would require multiple install commands
    // or modifying package.json temporarily
    
    let skip_set: HashSet<&str> = skip_deps.iter().map(|s| s.as_str()).collect();
    
    // Check if any required dependencies are in the skip list
    let has_skipped_build_deps = required_deps.iter()
        .any(|dep| skip_set.contains(dep.as_str()));
    
    if has_skipped_build_deps {
        // User is explicitly skipping build dependencies
        // This might break the build, but respect their choice
        match package_manager {
            "pnpm" => "pnpm install --prod".to_string(),
            "yarn" => "yarn install --production".to_string(),
            _ => "npm install --production".to_string(),
        }
    } else if !required_deps.is_empty() || !include_deps.is_empty() {
        // We need some devDependencies, install all
        match package_manager {
            "pnpm" => "pnpm install".to_string(),
            "yarn" => "yarn install".to_string(),
            _ => "npm install".to_string(),
        }
    } else {
        // No build dependencies needed, production only
        match package_manager {
            "pnpm" => "pnpm install --prod".to_string(),
            "yarn" => "yarn install --production".to_string(),
            _ => "npm install --production".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_detect_typescript_dependency() {
        let package_json = json!({
            "scripts": {
                "build": "tsc --build"
            },
            "devDependencies": {
                "typescript": "^5.0.0"
            }
        });
        
        let deps = detect_build_dependencies(&package_json);
        assert!(deps.contains("typescript"));
    }
    
    #[test]
    fn test_detect_babel_dependencies() {
        let package_json = json!({
            "scripts": {
                "build": "babel src -d dist --presets=@babel/preset-env"
            }
        });
        
        let deps = detect_build_dependencies(&package_json);
        assert!(deps.contains("@babel/core"));
        assert!(deps.contains("@babel/cli"));
        assert!(deps.contains("@babel/preset-env"));
    }
    
    #[test]
    fn test_safe_to_skip() {
        assert!(is_safe_to_skip("jest"));
        assert!(is_safe_to_skip("eslint"));
        assert!(is_safe_to_skip("prettier"));
        assert!(is_safe_to_skip("@testing-library/react"));
        assert!(!is_safe_to_skip("typescript"));
        assert!(!is_safe_to_skip("webpack"));
    }
}