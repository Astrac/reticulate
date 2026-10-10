use std::{sync::Arc, time::Duration};

use comfy_table::{Attribute, Cell, CellAlignment, Table};
use metrics_lib::*;

const COUNTER_MESHING_MAKE_TRIANGLES: &str = "reticulate.meshing.make_triangles.calls";
const TIMER_MESHING_FILL_PROJECTED: &str = "reticulate.meshing.surface_walk.fill_projected";
const TIMER_MESHING_FILL_REGION: &str = "reticulate.meshing.surface_walk.fill_region";
const TIMER_MESHING_GATHERING_WALK: &str = "reticulate.meshing.gathering_walk";
const TIMER_MESHING_ITERATE_MATERIALS: &str = "reticulate.meshing.iterate_materials";
const TIMER_MESHING_MAKE_TRIANGLES: &str = "reticulate.meshing.make_triangles.timer";
const TIMER_MESHING_SCAN_PROJECTED: &str = "reticulate.meshing.surface_walk.scan_projected";
const TIMER_MESHING_SURFACE_WALK: &str = "reticulate.meshing.surface_walk";
const TIMER_MESHING_WEAVE_REGION: &str = "reticulate.meshing.weave_region";

pub fn init_metrics() {
    init();
    let m = metrics();
    m.counter(COUNTER_MESHING_MAKE_TRIANGLES);
    m.timer(TIMER_MESHING_FILL_REGION);
    m.timer(TIMER_MESHING_ITERATE_MATERIALS);
    m.timer(TIMER_MESHING_MAKE_TRIANGLES);
    m.timer(TIMER_MESHING_GATHERING_WALK);
    m.timer(TIMER_MESHING_SURFACE_WALK);
    m.timer(TIMER_MESHING_FILL_PROJECTED);
    m.timer(TIMER_MESHING_SCAN_PROJECTED);
    m.timer(TIMER_MESHING_WEAVE_REGION);
}

pub fn report_and_reset() {
    let m = metrics();
    let counter_make_triangles = m.counter(COUNTER_MESHING_MAKE_TRIANGLES);
    let timer_fill_projected = m.timer(TIMER_MESHING_FILL_PROJECTED);
    let timer_fill_region = m.timer(TIMER_MESHING_FILL_REGION);
    let timer_gathering_walk = m.timer(TIMER_MESHING_GATHERING_WALK);
    let timer_iterate_materials = m.timer(TIMER_MESHING_ITERATE_MATERIALS);
    let timer_make_triangles = m.timer(TIMER_MESHING_MAKE_TRIANGLES);
    let timer_scan_projected = m.timer(TIMER_MESHING_SCAN_PROJECTED);
    let timer_surface_walk = m.timer(TIMER_MESHING_SURFACE_WALK);
    let timer_weaving = m.timer(TIMER_MESHING_WEAVE_REGION);

    let make_triangles_calls = counter_make_triangles.get();

    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("Timing reports (ms.µs.ns)").add_attribute(Attribute::Bold),
            Cell::new("Minimum").set_alignment(CellAlignment::Center),
            Cell::new("Average").set_alignment(CellAlignment::Center),
            Cell::new("Maximum").set_alignment(CellAlignment::Center),
            Cell::new("Total avg per bench").set_alignment(CellAlignment::Center),
        ])
        .add_row(format_timer_row(
            &format!("Mesher::make_triangles ({make_triangles_calls} samples)"),
            &timer_make_triangles,
            make_triangles_calls,
        ))
        .add_row(format_timer_row(
            "Mesher::GatheringWalk",
            &timer_gathering_walk,
            make_triangles_calls,
        ))
        .add_row(format_timer_row(
            "Mesher::SurfaceWalk",
            &timer_surface_walk,
            make_triangles_calls,
        ))
        .add_row(format_timer_row(
            "Material::fill_projected",
            &timer_fill_projected,
            make_triangles_calls,
        ))
        .add_row(format_timer_row(
            "SurfaceScanner::scan_projected",
            &timer_scan_projected,
            make_triangles_calls,
        ))
        .add_row(format_timer_row(
            "Filler::fill_region",
            &timer_fill_region,
            make_triangles_calls,
        ))
        .add_row(format_timer_row(
            "Weaver::weave_region",
            &timer_weaving,
            make_triangles_calls,
        ))
        .add_row(format_timer_row(
            "Material::iterate_materials",
            &timer_iterate_materials,
            make_triangles_calls,
        ));

    println!("{table}");

    counter_make_triangles.reset();
    timer_fill_projected.reset();
    timer_fill_region.reset();
    timer_gathering_walk.reset();
    timer_iterate_materials.reset();
    timer_make_triangles.reset();
    timer_scan_projected.reset();
    timer_surface_walk.reset();
    timer_weaving.reset();
}

fn format_timer_row(name: &str, timer: &Arc<Timer>, make_triangles_calls: u64) -> Vec<Cell> {
    vec![
        Cell::new(name.to_owned()),
        Cell::new(format_duration_micros(timer.min())).set_alignment(CellAlignment::Right),
        Cell::new(format_duration_micros(timer.average())).set_alignment(CellAlignment::Right),
        Cell::new(format_duration_micros(timer.max())).set_alignment(CellAlignment::Right),
        Cell::new(format_duration_micros(
            timer.total() / make_triangles_calls as u32,
        ))
        .set_alignment(CellAlignment::Right),
    ]
}

fn format_duration_micros(duration: Duration) -> String {
    let subsec_millis = duration.subsec_millis();
    let subsec_micros = duration.subsec_micros();

    let millis = duration.as_millis();
    let micros = duration.subsec_micros() - (subsec_millis * 1000);
    let nanos = duration.subsec_nanos() - (subsec_micros * 1000);

    format!("{millis}.{micros:03}.{nanos:03}")
}

#[inline(always)]
pub fn counter_make_triangles() -> Arc<Counter> {
    metrics().counter(COUNTER_MESHING_MAKE_TRIANGLES)
}

#[inline(always)]
pub fn timer_make_triangles() -> Arc<Timer> {
    metrics().timer(TIMER_MESHING_MAKE_TRIANGLES)
}

#[inline(always)]
pub fn timer_gathering_walk() -> Arc<Timer> {
    metrics().timer(TIMER_MESHING_GATHERING_WALK)
}

#[inline(always)]
pub fn timer_surface_walk() -> Arc<Timer> {
    metrics().timer(TIMER_MESHING_SURFACE_WALK)
}

#[inline(always)]
pub fn timer_scan_projected() -> Arc<Timer> {
    metrics().timer(TIMER_MESHING_SCAN_PROJECTED)
}

#[inline(always)]
pub fn timer_fill_projected() -> Arc<Timer> {
    metrics().timer(TIMER_MESHING_FILL_PROJECTED)
}

#[inline(always)]
pub fn timer_iterate_materials() -> Arc<Timer> {
    metrics().timer(TIMER_MESHING_ITERATE_MATERIALS)
}

#[inline(always)]
pub fn timer_fill_region() -> Arc<Timer> {
    metrics().timer(TIMER_MESHING_FILL_REGION)
}

#[inline(always)]
pub fn timer_weave_region() -> Arc<Timer> {
    metrics().timer(TIMER_MESHING_WEAVE_REGION)
}
