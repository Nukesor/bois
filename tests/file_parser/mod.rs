use std::{fs::read_to_string, path::PathBuf};

use bois::state::file_parser::config_file;
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Go through a list of file examples and make sure that the parser properly detects the correct
/// blocks.
#[rstest]
pub fn test_at_start(#[files("tests/file_parser/input/*")] case: PathBuf) -> TestResult {
    use winnow::Parser;

    let input = read_to_string(&case)?;
    let output = config_file.parse(input.as_str())?;

    let name = case.file_name().unwrap().to_str().unwrap();

    // Run the tests with the input being displayed as the description.
    // Makes reviewing this whole stuff a lot easier.
    //
    // The snapshot names are enumerated to guarantee the correct order while reviewing.
    let input_clone = input.clone();
    insta::with_settings!({
        description => input_clone,
    }, {
        if let Some(pre_config_block) = output.pre_config_block {
            assert_snapshot!(format!("{name}_0_pre_content"), pre_config_block);
        }

        if let Some(config) = output.config_block {
            assert_snapshot!(format!("{name}_1_config"), config);
        }

        if let Some(post_config_block) = output.post_config_block {
            assert_snapshot!(format!("{name}_2_post_content"), post_config_block);
        }
    });

    Ok(())
}
