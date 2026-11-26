use cuakey_application::{PlacedWindow, WindowPlace, WindowPlaces, WindowPlacesStore as _};

use super::JsonWindowPlacesStore;
use crate::test_dir::TestDir;

const FILE_NAME: &str = "window-places.json";

fn place(x: f32, y: f32, scale: f32) -> WindowPlace {
    WindowPlace { x, y, scale }
}

#[tokio::test]
async fn window_places_round_trip() {
    let dir = TestDir::new("places_round_trip");
    let store = JsonWindowPlacesStore::new(dir.path().to_path_buf());
    let mut places = WindowPlaces::default();
    places.set(PlacedWindow::Settings, place(120.0, 80.0, 1.5));
    places.set(PlacedWindow::Macros, place(-1800.0, 200.0, 1.0));
    places.set(PlacedWindow::Convert, place(40.0, 60.0, 1.25));
    places.set(PlacedWindow::AlwaysOn, place(3000.0, -20.0, 2.0));

    store.save(&places).await.unwrap();

    assert_eq!(store.load().await.unwrap(), places, "each window keeps its own place");
    assert_eq!(
        dir.file_names(),
        [FILE_NAME],
        "one file of its own, the temporary one renamed over it"
    );
    let file: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.path().join(FILE_NAME)).unwrap()).unwrap();
    assert_eq!(
        file["always_on"],
        serde_json::json!({ "x": 3000.0, "y": -20.0, "scale": 2.0 }),
        "each window under its name: {file}"
    );
}

#[tokio::test]
async fn a_missing_window_places_file_gives_none() {
    let dir = TestDir::new("places_missing");
    let store = JsonWindowPlacesStore::new(dir.path().to_path_buf());

    assert_eq!(store.load().await.unwrap(), WindowPlaces::default());
    assert!(dir.file_names().is_empty(), "loading writes nothing");
}

#[tokio::test]
async fn a_broken_window_places_file_gives_none() {
    let dir = TestDir::new("places_broken");
    let store = JsonWindowPlacesStore::new(dir.path().to_path_buf());
    for broken in ["", "{ \"settings\": ", "[1, 2]", "\"settings\""] {
        std::fs::write(dir.path().join(FILE_NAME), broken).unwrap();

        let loaded = store.load().await;

        assert_eq!(
            loaded.unwrap(),
            WindowPlaces::default(),
            "{broken:?} is no error, only no place"
        );
    }
}

#[tokio::test]
async fn an_unknown_window_places_entry_gives_none_for_it_alone() {
    let dir = TestDir::new("places_unknown_entry");
    let store = JsonWindowPlacesStore::new(dir.path().to_path_buf());
    let text = r#"{
        "settings": { "x": 120, "y": 80, "scale": 1.5 },
        "macros": "by the window",
        "convert": { "x": 40 },
        "toolbar": { "x": 0, "y": 0, "scale": 1 }
    }"#;
    std::fs::write(dir.path().join(FILE_NAME), text).unwrap();

    let mut expected = WindowPlaces::default();
    expected.set(PlacedWindow::Settings, place(120.0, 80.0, 1.5));
    assert_eq!(store.load().await.unwrap(), expected);
}
