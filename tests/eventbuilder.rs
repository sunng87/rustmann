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

    assert_eq!(123, event.get_time());
    assert_eq!("ok", event.get_state());
    assert_eq!("test_service", event.get_service());
    assert_eq!("localhost", event.get_host());
    assert_eq!("short desc", event.get_description());
    assert_eq!(2, event.get_tags().len());
    assert_eq!(5.0, event.get_ttl());
    assert_eq!(123000, event.get_time_micros());
    assert_eq!(100, event.get_metric_sint64());
    assert_eq!(1.0, event.get_metric_d());
    assert_eq!(2.0, event.get_metric_f());
    assert_eq!(1, event.get_attributes().len());
}
