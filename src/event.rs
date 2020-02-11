use crate::protos::riemann::{Attribute, Event};

/// Riemann event data builder
#[derive(Default)]
pub struct EventBuilder {
    result: Event,
}

impl EventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn time(mut self, time: i64) -> Self {
        self.result.time = Some(time);
        self
    }

    pub fn state<S: Into<String>>(mut self, state: S) -> Self {
        self.result.state = Some(state.into());
        self
    }

    pub fn service<S: Into<String>>(mut self, service: S) -> Self {
        self.result.service = Some(service.into());
        self
    }

    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.result.host = Some(host.into());
        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.result.description = Some(description.into());
        self
    }

    pub fn add_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.result.tags.push(tag.into());
        self
    }

    pub fn ttl(mut self, ttl: f32) -> Self {
        self.result.ttl = Some(ttl);
        self
    }

    pub fn time_micros(mut self, time_micros: i64) -> Self {
        self.result.time_micros = Some(time_micros);
        self
    }

    pub fn metric_sint64(mut self, metric_sint64: i64) -> Self {
        self.result.metric_sint64 = Some(metric_sint64);
        self
    }

    pub fn metric_d(mut self, metric_d: f64) -> Self {
        self.result.metric_d = Some(metric_d);
        self
    }

    pub fn metric_f(mut self, metric_f: f32) -> Self {
        self.result.metric_f = Some(metric_f);
        self
    }

    pub fn add_attribute<S: Into<String>>(mut self, key: S, value: Option<S>) -> Self {
        let attr = Attribute {
            key: key.into(),
            value: value.map(|v| v.into()),
        };
        self.result.attributes.push(attr);
        self
    }

    pub fn build(self) -> Event {
        self.result
    }
}
