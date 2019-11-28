use crate::protos::riemann::{Attribute, Event};

// use protobuf::Chars;

/// Riemann event data builder
#[derive(Default)]
pub struct EventBuilder {
    time: Option<i64>,
    state: Option<String>,
    service: Option<String>,
    host: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    ttl: Option<f32>,

    time_micros: Option<i64>,
    metric_sint64: Option<i64>,
    metric_d: Option<f64>,
    metric_f: Option<f32>,

    attributes: Vec<Attribute>,
}

impl EventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn time(mut self, time: i64) -> Self {
        self.time = Some(time);
        self
    }

    pub fn state<S: Into<String>>(mut self, state: S) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn service<S: Into<String>>(mut self, service: S) -> Self {
        self.service = Some(service.into());
        self
    }

    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn add_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn ttl(mut self, ttl: f32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn time_micros(mut self, time_micros: i64) -> Self {
        self.time_micros = Some(time_micros);
        self
    }

    pub fn metric_sint64(mut self, metric_sint64: i64) -> Self {
        self.metric_sint64 = Some(metric_sint64);
        self
    }

    pub fn metric_d(mut self, metric_d: f64) -> Self {
        self.metric_d = Some(metric_d);
        self
    }

    pub fn metric_f(mut self, metric_f: f32) -> Self {
        self.metric_f = Some(metric_f);
        self
    }

    pub fn add_attribute<S: Into<String>>(mut self, key: S, value: Option<S>) -> Self {
        let mut attr = Attribute::new();
        attr.set_key(key.into());
        if let Some(value) = value {
            attr.set_value(value.into());
        }
        self.attributes.push(attr);
        self
    }

    pub fn build(self) -> Event {
        let mut event = Event::new();

        if let Some(time) = self.time {
            event.set_time(time);
        }

        if let Some(state) = self.state {
            event.set_state(state);
        }

        if let Some(service) = self.service {
            event.set_service(service);
        }

        if let Some(host) = self.host {
            event.set_host(host);
        }

        if let Some(description) = self.description {
            event.set_description(description);
        }

        event.set_tags(self.tags.iter().map(|t| t.into()).collect());

        if let Some(ttl) = self.ttl {
            event.set_ttl(ttl);
        }

        if let Some(time_micros) = self.time_micros {
            event.set_time_micros(time_micros);
        }

        if let Some(metric_sint64) = self.metric_sint64 {
            event.set_metric_sint64(metric_sint64);
        }

        if let Some(metric_d) = self.metric_d {
            event.set_metric_d(metric_d);
        }

        if let Some(metric_f) = self.metric_f {
            event.set_metric_f(metric_f);
        }

        event.set_attributes(self.attributes.into());

        event
    }
}
