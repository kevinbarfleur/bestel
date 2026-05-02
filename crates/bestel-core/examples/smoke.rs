use bestel_core::pob::locator::find_pob_dirs;
use bestel_core::pob::parser::parse_file;
use bestel_core::pob::watcher::PobWatcher;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let installs = find_pob_dirs();
    println!("=== PoB installs detected ===");
    for i in &installs {
        println!("  [{}] {}", i.game.label(), i.builds_dir.display());
    }
    println!();

    let watcher = PobWatcher::start()?;
    if let Some(b) = watcher.initial_build() {
        println!("=== Latest build ===");
        println!("  source : {}", b.source_file.display());
        println!("  game   : {}", b.game.label());
        println!("  class  : {}", b.class);
        println!("  asc    : {:?}", b.ascendancy);
        println!("  level  : {:?}", b.level);
        println!("  target : {:?}", b.target_version);
        println!("  main   : {:?}", b.main_skill);
        println!(
            "  life   : {:?}  mana: {:?}  ES: {:?}  EHP: {:?}",
            b.life(),
            b.mana(),
            b.energy_shield(),
            b.ehp()
        );
        println!("  DPS    : {:?}", b.dps());
        println!("  res    : {:?}", b.resistances());
        println!("  items  : {} item(s)", b.items.len());
        println!(
            "  groups : {} skill group(s) — main: {:?}",
            b.skill_groups.len(),
            b.skill_groups.iter().find(|g| g.is_main).map(|g| &g.label)
        );
        println!("  notes  : {}", &b.notes.chars().take(80).collect::<String>());
    } else {
        println!("(no build found)");
    }

    println!();
    println!("=== Parse every detected XML ===");
    for install in &installs {
        if let Ok(rd) = std::fs::read_dir(&install.builds_dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                    match parse_file(&path) {
                        Ok(b) => println!(
                            "  OK  {} → {} {} lvl {:?}",
                            path.file_name().unwrap().to_string_lossy(),
                            b.game.label(),
                            b.class,
                            b.level
                        ),
                        Err(e) => println!(
                            "  ERR {} → {}",
                            path.file_name().unwrap().to_string_lossy(),
                            e
                        ),
                    }
                }
            }
        }
    }

    drop(watcher);
    Ok(())
}
