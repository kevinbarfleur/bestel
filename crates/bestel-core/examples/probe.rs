use bestel_core::llm::detect::detect_provider;

#[tokio::main]
async fn main() {
    let d = detect_provider().await;
    println!("provider: {:?}", d.provider.as_ref().map(|p| p.label()));
    for p in &d.probes {
        println!("  [{}] {} v={:?} note={:?}", if p.installed {"+"} else {" "}, p.name, p.version, p.note);
    }
}
