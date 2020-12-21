/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::defer_stream::DeferStreamInterface;
use common::NamedItem;
use graphql_ir::Selection;
use graphql_ir::{FragmentSpread, InlineFragment, Program, Transformed, Transformer};
use std::iter;
use std::sync::Arc;

/// Transform to unwrap selections wrapped in a InlineFragment with custom
/// directive for printing
pub fn unwrap_custom_directive_selection(
    program: &Program,
    defer_stream_interface: &DeferStreamInterface,
) -> Program {
    let mut transform = UnwrapCustomDirectiveSelection::new(defer_stream_interface);
    transform
        .transform_program(program)
        .replace_or_else(|| program.clone())
}

struct UnwrapCustomDirectiveSelection<'s> {
    defer_stream_interface: &'s DeferStreamInterface,
}

impl<'s> UnwrapCustomDirectiveSelection<'s> {
    fn new(defer_stream_interface: &'s DeferStreamInterface) -> Self {
        Self {
            defer_stream_interface,
        }
    }
}

impl Transformer for UnwrapCustomDirectiveSelection<'_> {
    const NAME: &'static str = "UnwrapCustomDirectiveSelection";
    const VISIT_ARGUMENTS: bool = false;
    const VISIT_DIRECTIVES: bool = false;

    fn transform_inline_fragment(&mut self, fragment: &InlineFragment) -> Transformed<Selection> {
        if fragment.type_condition.is_none() {
            // Remove the wrapping `... @defer` for `@defer` on fragment spreads.
            let defer = fragment
                .directives
                .named(self.defer_stream_interface.defer_name);
            if let Some(defer) = defer {
                if let Selection::FragmentSpread(frag_spread) = &fragment.selections[0] {
                    return Transformed::Replace(Selection::FragmentSpread(Arc::new(
                        FragmentSpread {
                            directives: frag_spread
                                .directives
                                .iter()
                                .chain(iter::once(defer))
                                .cloned()
                                .collect(),
                            fragment: frag_spread.fragment,
                            arguments: frag_spread.arguments.clone(),
                        },
                    )));
                }
            }
        }
        self.default_transform_inline_fragment(fragment)
    }
}
