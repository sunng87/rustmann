use rustmann::EventBuilder;

#[test]
fn test_builder() {
    let event = EventBuilder::new()
        .time(123)
        .state("ok")
        .service("test_service")
        .host("localhost")
        .description("short desc")
        .add_tag("tag1")
        .add_tag("tag2")
        .ttl(5.0)
        .time_micros(123000)
        .metric_sint64(100)
        .metric_d(1.0)
        .metric_f(2.0)
        .add_attribute("name", Some("Joe"))
        .build();

    assert_eq!(123, event.time.unwrap());
    assert_eq!("ok", event.state.unwrap());
    assert_eq!("test_service", event.service.unwrap());
    assert_eq!("localhost", event.host.unwrap());
    assert_eq!("short desc", event.description.unwrap());
    assert_eq!(2, event.tags.len());
    assert_eq!(5.0, event.ttl.unwrap());
    assert_eq!(123000, event.time_micros.unwrap());
    assert_eq!(100, event.metric_sint64.unwrap());
    assert_eq!(1.0, event.metric_d.unwrap());
    assert_eq!(2.0, event.metric_f.unwrap());
    assert_eq!(1, event.attributes.len());
}
