// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use inherent::inherent;
use std::sync::Arc;

use super::{CommonMetricData, MetricId};
use crate::ipc::need_ipc;

/// A text metric.
///
/// Record a string value with arbitrary content. Supports non-ASCII
/// characters.
///
/// # Example
///
/// The following piece of code will be generated by `glean_parser`:
///
/// ```rust,ignore
/// use glean::metrics::{TextMetric, CommonMetricData, Lifetime};
/// use once_cell::sync::Lazy;
///
/// mod browser {
///     pub static bread_recipe: Lazy<TextMetric> = Lazy::new(|| TextMetric::new(CommonMetricData {
///         name: "bread_recipe".into(),
///         category: "browser".into(),
///         lifetime: Lifetime::Ping,
///         disabled: false,
///         dynamic_label: None
///     }));
/// }
/// ```
///
/// It can then be used with:
///
/// ```rust,ignore
/// browser::bread_recipe.set("The 'baguette de tradition française' is made from wheat flour, water, yeast, and common salt. It may contain up to 2% broad bean flour, up to 0.5% soya flour, and up to 0.3% wheat malt flour.");
/// ```

#[derive(Clone)]
pub enum TextMetric {
    Parent(Arc<glean::private::TextMetric>),
    Child(TextMetricIpc),
}

#[derive(Clone, Debug)]
pub struct TextMetricIpc;

impl TextMetric {
    /// Create a new text metric.
    pub fn new(_id: MetricId, meta: CommonMetricData) -> Self {
        if need_ipc() {
            TextMetric::Child(TextMetricIpc)
        } else {
            TextMetric::Parent(Arc::new(glean::private::TextMetric::new(meta)))
        }
    }

    #[cfg(test)]
    pub(crate) fn child_metric(&self) -> Self {
        match self {
            TextMetric::Parent(_) => TextMetric::Child(TextMetricIpc),
            TextMetric::Child(_) => panic!("Can't get a child metric from a child process"),
        }
    }
}

#[inherent]
impl glean::traits::Text for TextMetric {
    /// Sets to the specified value.
    ///
    /// # Arguments
    ///
    /// * `value` - The text to set the metric to.
    pub fn set<S: Into<std::string::String>>(&self, value: S) {
        match self {
            TextMetric::Parent(p) => {
                p.set(value.into());
            }
            TextMetric::Child(_) => {
                log::error!("Unable to set text metric in non-main process. Ignoring.")
            }
        }
    }

    /// **Exported for test purposes.**
    ///
    /// Gets the currently stored value as a string.
    ///
    /// This doesn't clear the stored value.
    ///
    /// # Arguments
    ///
    /// * `ping_name` - represents the optional name of the ping to retrieve the
    ///   metric for. Defaults to the first value in `send_in_pings`.
    pub fn test_get_value<'a, S: Into<Option<&'a str>>>(
        &self,
        ping_name: S,
    ) -> Option<std::string::String> {
        let ping_name = ping_name.into().map(|s| s.to_string());
        match self {
            TextMetric::Parent(p) => p.test_get_value(ping_name),
            TextMetric::Child(_) => {
                panic!("Cannot get test value for text metric in non-parent process!")
            }
        }
    }

    /// **Exported for test purposes.**
    ///
    /// Gets the number of recorded errors for the given metric and error type.
    ///
    /// # Arguments
    ///
    /// * `error` - The type of error
    /// * `ping_name` - represents the optional name of the ping to retrieve the
    ///   metric for. Defaults to the first value in `send_in_pings`.
    ///
    /// # Returns
    ///
    /// The number of errors reported.
    pub fn test_get_num_recorded_errors(&self, error: glean::ErrorType) -> i32 {
        match self {
            TextMetric::Parent(p) => p.test_get_num_recorded_errors(error),
            TextMetric::Child(_) => panic!(
                "Cannot get the number of recorded errors for text metric in non-parent process!"
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{common_test::*, ipc, metrics};

    #[test]
    fn sets_text_value() {
        let _lock = lock_test();

        let metric = &metrics::test_only_ipc::a_text;

        metric.set("test_text_value");

        assert_eq!("test_text_value", metric.test_get_value("store1").unwrap());
    }

    #[test]
    fn text_ipc() {
        // TextMetric doesn't support IPC.
        let _lock = lock_test();

        let parent_metric = &metrics::test_only_ipc::a_text;

        parent_metric.set("test_parent_value");

        {
            let child_metric = parent_metric.child_metric();

            let _raii = ipc::test_set_need_ipc(true);

            // Instrumentation calls do not panic.
            child_metric.set("test_child_value");

            // (They also shouldn't do anything,
            // but that's not something we can inspect in this test)
        }

        assert!(ipc::replay_from_buf(&ipc::take_buf().unwrap()).is_ok());

        assert!(
            "test_parent_value" == parent_metric.test_get_value("store1").unwrap(),
            "Text metrics should only work in the parent process"
        );
    }
}