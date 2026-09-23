use desktop_mcp::utils::coordinates::{CoordinateTransformer, Point, Rect};

#[test]
fn test_retina_coordinate_scaling() {
    // 2x Retina display
    let transformer = CoordinateTransformer::new(2.0);

    let logical = Point { x: 150.0, y: 250.0 };
    let physical = transformer.to_physical(&logical);

    assert_eq!(physical.x, 300.0);
    assert_eq!(physical.y, 500.0);

    let back_to_logical = transformer.to_logical(&physical);
    assert_eq!(back_to_logical.x, 150.0);
    assert_eq!(back_to_logical.y, 250.0);
}

#[test]
fn test_window_relative_coordinates() {
    let window_rect = Rect {
        x: 200.0,
        y: 100.0,
        width: 800.0,
        height: 600.0,
    };

    // Point in window relative coordinates (50, 50)
    let rel_point = Point { x: 50.0, y: 50.0 };
    let screen_point = window_rect.to_screen_coordinates(&rel_point);

    assert_eq!(screen_point.x, 250.0);
    assert_eq!(screen_point.y, 150.0);

    // Assert containment
    assert!(window_rect.contains(250.0, 150.0));
    assert!(!window_rect.contains(100.0, 50.0));
}
