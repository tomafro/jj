// Copyright 2024 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use jj_cli::cli_util::CliRunner;
use jj_cli::operation_templater::{
    OperationTemplateBuildFnTable, OperationTemplateLanguageExtension,
};
use jj_cli::template_builder::TemplateLanguage;
use jj_cli::template_parser::{self, TemplateParseError};
use jj_cli::templater::{TemplateFunction, TemplatePropertyError};
use jj_lib::extensions_map::ExtensionsMap;
use jj_lib::object_id::ObjectId;
use jj_lib::op_store::OperationId;
use jj_lib::operation::Operation;

struct HexCounter;

fn num_digits_in_id(id: &OperationId) -> i64 {
    let mut count = 0;
    for ch in id.hex().chars() {
        if ch.is_ascii_digit() {
            count += 1;
        }
    }
    count
}

fn num_char_in_id(operation: Operation, ch_match: char) -> Result<i64, TemplatePropertyError> {
    let mut count = 0;
    for ch in operation.id().hex().chars() {
        if ch == ch_match {
            count += 1;
        }
    }
    Ok(count)
}

impl OperationTemplateLanguageExtension for HexCounter {
    fn build_fn_table(&self) -> OperationTemplateBuildFnTable {
        let mut table = OperationTemplateBuildFnTable::empty();
        table.operation_methods.insert(
            "num_digits_in_id",
            |language, _build_context, property, call| {
                template_parser::expect_no_arguments(call)?;
                Ok(
                    language.wrap_integer(TemplateFunction::new(property, |operation| {
                        Ok(num_digits_in_id(operation.id()))
                    })),
                )
            },
        );
        table.operation_methods.insert(
            "num_char_in_id",
            |language, _build_context, property, call| {
                let [string_arg] = template_parser::expect_exact_arguments(call)?;
                let char_arg =
                    template_parser::expect_string_literal_with(string_arg, |string, span| {
                        let chars: Vec<_> = string.chars().collect();
                        match chars[..] {
                            [ch] => Ok(ch),
                            _ => Err(TemplateParseError::unexpected_expression(
                                "Expected singular character argument",
                                span,
                            )),
                        }
                    })?;

                Ok(
                    language.wrap_integer(TemplateFunction::new(property, move |operation| {
                        num_char_in_id(operation, char_arg)
                    })),
                )
            },
        );

        table
    }

    fn build_cache_extensions(&self, _extensions: &mut ExtensionsMap) {}
}

fn main() -> std::process::ExitCode {
    CliRunner::init()
        .set_operation_template_extension(Box::new(HexCounter))
        .run()
}
