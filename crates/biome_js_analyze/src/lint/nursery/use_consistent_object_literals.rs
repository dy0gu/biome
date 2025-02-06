use biome_analyze::{
    context::RuleContext, declare_lint_rule, Ast, Rule, RuleDiagnostic, RuleSource, RuleSourceKind,
};
use biome_console::markup;
use biome_deserialize_macros::Deserializable;
use biome_js_syntax::{inner_string_text, AnyJsExpression, AnyJsObjectMember};
use biome_rowan::AstNode;
use serde::{Deserialize, Serialize};

declare_lint_rule! {
    /// Require consistently using either explicit object property assignment or the newer shorthand syntax, when possible.
    ///
    /// ECMAScript 6 provides two ways to define an object literal: `{foo: foo}` and `{foo}`.
    /// The two styles are functionally equivalent.
    /// Using the same style consistently across your codebase makes it easier to quickly read and understand object definitions.
    ///
    /// ## Example
    ///
    /// ### Invalid
    /// ```js,expect_diagnostic
    /// let foo = 1;
    /// let invalid = {
    ///     foo
    /// };
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// let invalid = {
    ///     bar() { return "bar"; },
    /// };
    /// ```
    ///
    /// ### Valid
    /// ```js
    /// let foo = 1;
    /// let valid = {
    ///     foo: foo,
    ///     bar: function() { return "bar"; },
    ///     arrow: () => { "arrow" },
    ///     get getter() { return "getter"; },
    ///     set setter(value) { this._setter = value; }
    /// };
    /// ```
    ///
    /// ## Options
    ///
    /// Use the options to specify the syntax of object literals to enforce.
    ///
    /// ```json,options
    /// {
    ///     "options": {
    ///         "syntax": "explicit"
    ///     }
    /// }
    /// ```
    ///
    /// ### syntax
    ///
    /// The syntax to use:
    /// - `explicit`: enforces the use of explicit object property syntax in every case
    /// - `shorthand`: enforces the use of shorthand object property syntax when possible
    ///
    /// Default: `explicit`
    pub UseConsistentObjectLiterals {
        version: "next",
        name: "useConsistentObjectLiterals",
        language: "js",
        recommended: false,
        sources: &[RuleSource::Eslint("object-shorthand")],
        source_kind: RuleSourceKind::Inspired,
    }
}

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentObjectLiteralsOptions {
    syntax: ObjectLiteralSyntax,
}

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum ObjectLiteralSyntax {
    #[default]
    Explicit,
    Shorthand,
}

impl Rule for UseConsistentObjectLiterals {
    type Query = Ast<AnyJsObjectMember>;
    type State = String;
    type Signals = Option<Self::State>;
    type Options = UseConsistentObjectLiteralsOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let binding = ctx.query();
        let options = ctx.options();
        match binding {
            AnyJsObjectMember::JsShorthandPropertyObjectMember(source) => {
                let member_token = source.name().ok()?.value_token().ok()?;
                let member_id = inner_string_text(&member_token);

                match options.syntax {
                    ObjectLiteralSyntax::Shorthand => None,
                    ObjectLiteralSyntax::Explicit => Some(member_id.to_string()),
                }
            }
            AnyJsObjectMember::JsPropertyObjectMember(source) => {
                let member_token = source
                    .name()
                    .ok()?
                    .as_js_literal_member_name()?
                    .value()
                    .ok()?;
                let member_id = inner_string_text(&member_token);
                let reference_token = source.value().ok()?;
                let reference_id = match reference_token {
                    AnyJsExpression::JsIdentifierExpression(identifier) => {
                        let variable_token = identifier.name().ok()?.value_token().ok()?;
                        inner_string_text(&variable_token)
                    }
                    AnyJsExpression::JsCallExpression(call) => {
                        let callee_token = call
                            .callee()
                            .ok()?
                            .as_js_identifier_expression()?
                            .name()
                            .ok()?
                            .value_token()
                            .ok()?;
                        inner_string_text(&callee_token)
                    }
                    AnyJsExpression::JsFunctionExpression(function) => {
                        let function_token = function.function_token().ok()?;
                        inner_string_text(&function_token)
                    }
                    _ => return None,
                };

                match options.syntax {
                    ObjectLiteralSyntax::Shorthand => {
                        if member_id == reference_id || "function" == reference_id {
                            Some(member_id.to_string())
                        } else {
                            None
                        }
                    }
                    ObjectLiteralSyntax::Explicit => None,
                }
            }
            AnyJsObjectMember::JsMethodObjectMember(source) => {
                let member_token = source
                    .name()
                    .ok()?
                    .as_js_literal_member_name()?
                    .value()
                    .ok()?;
                let member_id = inner_string_text(&member_token);

                match options.syntax {
                    ObjectLiteralSyntax::Shorthand => None,
                    ObjectLiteralSyntax::Explicit => Some(member_id.to_string()),
                }
            }
            _ => None,
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();
        let options = ctx.options();

        let title = match options.syntax {
            ObjectLiteralSyntax::Shorthand => {
                "Use shorthand object property syntax whenever possible."
            }
            ObjectLiteralSyntax::Explicit => {
                "Always use explicit object property assignment syntax."
            }
        };

        Some(
            RuleDiagnostic::new(rule_category!(), node.range(), markup! {{title}}).note(
                markup! { "Using a consistent object literal syntax makes it easier to anticipate the structure of an object." },
            ),
        )
    }
}
