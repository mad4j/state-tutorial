use state_tutorial::{Component, ComponentInterface, ConfigParams};

fn main() {
    println!("=== ESSOR/SCA Component State Machine Demo ===\n");

    let mut c = Component::new("radio-component");

    // Step 1 – Query initial state
    let status = c.query();
    println!("[query]  {}", status.description);

    // Step 2 – First config call: Inactive → Loaded
    c.config(ConfigParams::new().with("frequency", "100MHz"))
        .expect("config (Inactive→Loaded) failed");
    let status = c.query();
    println!("[config] {}", status.description);

    // Step 3 – Self-test in Loaded state
    let result = c.test().expect("test failed");
    println!("[test]   passed={}, details={}", result.passed, result.details);

    // Step 4 – Second config call: Loaded → Ready
    c.config(ConfigParams::new().with("mode", "active").with("power", "high"))
        .expect("config (Loaded→Ready) failed");
    let status = c.query();
    println!("[config] {}", status.description);

    // Step 5 – Self-test in Ready state
    let result = c.test().expect("test failed");
    println!("[test]   passed={}, details={}", result.passed, result.details);

    // Step 6 – Start: Ready → Running
    c.start().expect("start failed");
    let status = c.query();
    println!("[start]  {}", status.description);

    // Step 7 – Stop: Running → Ready
    c.stop().expect("stop failed");
    let status = c.query();
    println!("[stop]   {}", status.description);

    // Step 8 – Reset: Ready → Inactive
    c.reset().expect("reset failed");
    let status = c.query();
    println!("[reset]  {}", status.description);

    println!("\nDemo complete.");
}
