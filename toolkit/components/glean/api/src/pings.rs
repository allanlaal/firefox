// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! This file contains the Generated Glean Metrics API (Ping portion)
//!
//! The contents of this module are generated by
//! `toolkit/components/glean/build_scripts/glean_parser_ext/run_glean_parser.py`, from
//! 'toolkit/components/glean/pings.yaml`.

include!(mozbuild::objdir_path!(
    "toolkit/components/glean/api/src/pings.rs"
));