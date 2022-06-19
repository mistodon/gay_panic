#[test]
#[should_panic] // NOTE: Comment out to view output
fn test_panic() {
    gay_panic::init_with(gay_panic::Config {
        call_previous_hook: false,
        force_capture_backtrace: true,
    });

    let x: Option<()> = None;
    x.map(|_| std::fs::read_to_string("").unwrap()).unwrap();
}
