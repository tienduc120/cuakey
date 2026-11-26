use super::{
    LoadWindowPlaces, LoadWindowPlacesError, PlacedWindow, SaveWindowPlaces, SaveWindowPlacesError, WindowPlace,
    WindowPlaces,
};
use crate::StoreError;
use crate::testing::{FakeWindowPlacesStore, Health};

fn place(x: f32, y: f32, scale: f32) -> WindowPlace {
    WindowPlace { x, y, scale }
}

#[test]
fn each_window_keeps_its_own_place() {
    let mut places = WindowPlaces::default();
    places.set(PlacedWindow::Settings, place(120.0, 80.0, 1.5));
    places.set(PlacedWindow::AlwaysOn, place(-1800.0, 200.0, 1.0));
    places.set(PlacedWindow::Settings, place(130.0, 90.0, 1.25));

    assert_eq!(
        places.get(PlacedWindow::Settings),
        Some(place(130.0, 90.0, 1.25)),
        "the last place wins"
    );
    assert_eq!(places.get(PlacedWindow::AlwaysOn), Some(place(-1800.0, 200.0, 1.0)));
    assert_eq!(
        places.get(PlacedWindow::Macros),
        None,
        "a window that never closed has none"
    );
    assert_eq!(
        places.iter().collect::<Vec<_>>(),
        [
            (PlacedWindow::Settings, place(130.0, 90.0, 1.25)),
            (PlacedWindow::AlwaysOn, place(-1800.0, 200.0, 1.0)),
        ]
    );
}

#[tokio::test]
async fn load_gives_the_stored_places() {
    let mut places = WindowPlaces::default();
    places.set(PlacedWindow::Macros, place(300.0, 120.0, 1.5));

    let loaded = LoadWindowPlaces::new(FakeWindowPlacesStore::holding(places.clone()))
        .execute()
        .await
        .unwrap();

    assert_eq!(loaded, places);
}

#[tokio::test]
async fn save_stores_the_places() {
    let mut places = WindowPlaces::default();
    places.set(PlacedWindow::Convert, place(40.0, 60.0, 2.0));
    let save = SaveWindowPlaces::new(FakeWindowPlacesStore::new(Health::Working));

    save.execute(&places).await.unwrap();

    assert_eq!(save.store().stored(), places);
}

#[tokio::test]
async fn a_storage_failure_keeps_its_cause() {
    let load = LoadWindowPlaces::new(FakeWindowPlacesStore::new(Health::Broken))
        .execute()
        .await;
    assert!(
        matches!(load, Err(LoadWindowPlacesError::Storage(StoreError::Storage(_)))),
        "{load:?}"
    );

    let save = SaveWindowPlaces::new(FakeWindowPlacesStore::new(Health::Broken))
        .execute(&WindowPlaces::default())
        .await;
    assert!(
        matches!(save, Err(SaveWindowPlacesError::Storage(StoreError::Storage(_)))),
        "{save:?}"
    );
}
