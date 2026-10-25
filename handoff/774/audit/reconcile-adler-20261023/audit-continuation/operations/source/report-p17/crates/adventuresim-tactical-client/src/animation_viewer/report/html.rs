//! Human review presentation.

use super::*;

pub(super) fn review_html(
    manifest: &AnimationCaptureManifest,
) -> Result<String, serde_json::Error> {
    let frame_json = serde_json::to_string(&manifest.frames)?;
    let scenario_names_json = serde_json::to_string(
        &manifest
            .scenarios
            .iter()
            .map(|scenario| scenario.scenario.label())
            .collect::<Vec<_>>(),
    )?;
    let quality_summary = format!(
        "Quality score: {} ({}/31 weighted defect points); acceptance: {}",
        describe_observation(&manifest.quality_score.quality_percent, |quality| format!("{:.2}%", quality.percent())),
        manifest.quality_score.weighted_defect_score,
        if manifest.quality_score.acceptance_passed {
            "passed"
        } else {
            "failed"
        },
    );
    let scenario_buttons = manifest
        .scenarios
        .iter()
        .map(|scenario| {
            format!(
                "<button data-scenario=\"{}\">{}</button>",
                scenario.scenario.label(),
                scenario.scenario.label()
            )
        })
        .collect::<String>();
    let metrics = manifest.scenarios.iter().map(scenario_measurements_html)
        .collect::<Result<Vec<_>, _>>()?.join("");
    let height_metrics = height_metrics_html(manifest);
    Ok(format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>Animation review</title><style>
body{{font:15px system-ui;background:#111820;color:#e8eef5;margin:24px}}button,select{{margin:4px;padding:8px}}img{{max-width:min(960px,100%);background:#222}}table{{border-collapse:collapse;margin-top:20px}}td,th{{border:1px solid #526171;padding:6px}}.note{{max-width:960px;color:#b9c7d5}}#contact{{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px;margin-top:20px}}#contact img{{width:100%}}
</style></head><body><h1>Tactical locomotion review</h1>
<p class="note">{quality_summary}</p><p class="note">This runs the shared tactical player, hills scene, gameplay camera, 64 Hz authoritative locomotion projection, authored FK, and final procedural passes. Gameplay images are raw; side/front diagnostics add the cyan skeleton and support markers. Use normal speed first, then slow motion.</p>
<div>{scenario_buttons}</div><label>View <select id="view"><option value="gameplay">gameplay (raw)</option><option value="side">side diagnostic</option><option value="front">front diagnostic</option></select></label>
<label>Playback <select id="rate"><option value="1">normal</option><option value="2">half speed</option><option value="4">quarter speed</option></select></label>
<p id="telemetry"></p><img id="player"><div id="contact"></div>
{metrics}
{height_metrics}
<script>const all={frame_json},scenarioNames={scenario_names_json};let scenario=scenarioNames[0]||"",i=0,timer;const player=document.querySelector('#player'),view=document.querySelector('#view'),rate=document.querySelector('#rate'),telemetry=document.querySelector('#telemetry');
function frames(){{return all.filter(x=>x.scenario===scenario)}}function show(){{const list=frames(),f=list.length?list[i%list.length]:null;if(!f){{player.removeAttribute('src');telemetry.textContent='No completed capture frames';return}}player.src=f.screenshots[view.value];telemetry.textContent=`${{f.scenario}} frame ${{f.scenario_frame}} | guard ${{f.weapon_guard}} lead ${{f.lead_foot}} | ${{f.speed_metres_per_second.toFixed(2)}} m/s | phase ${{f.gait_phase.toFixed(3)}} | world plants L ${{f.left_support_weight.toFixed(2)}} R ${{f.right_support_weight.toFixed(2)}}`;}}
function play(){{clearInterval(timer);timer=setInterval(()=>{{i=(i+1)%frames().length;show()}},1000/64*Number(rate.value))}}function contacts(){{const f=frames(),step=Math.max(1,Math.floor(f.length/12)),box=document.querySelector('#contact');box.innerHTML='';for(let n=0;n<f.length;n+=step){{let x=document.createElement('img');x.src=f[n].screenshots[view.value];x.title=`frame ${{f[n].scenario_frame}} phase ${{f[n].gait_phase.toFixed(3)}}`;box.appendChild(x)}}}}
document.querySelectorAll('button').forEach(b=>b.onclick=()=>{{scenario=b.dataset.scenario;i=0;show();contacts();play()}});view.onchange=()=>{{show();contacts()}};rate.onchange=play;show();contacts();play();</script></body></html>"#
    ))
}

fn describe<T>(
    worst: &Option<ContinuityLocation<T>>,
    measured: impl FnOnce(&T) -> String,
) -> Result<String, serde_json::Error> {
    let Some(worst) = worst else { return Ok("none".to_owned()); };
    let label = report_bone_label(worst.bone).ok_or_else(|| {
        <serde_json::Error as serde::ser::Error>::custom("uncaptured semantic bone in HTML report")
    })?;
    Ok(format!("{} {}&rarr;{} ({})", label, worst.from_frame, worst.to_frame, measured(&worst.value)))
}

fn scenario_measurements_html(metrics: &ScenarioMetrics) -> Result<String, serde_json::Error> {
    let continuity = match &metrics.continuity {
        Observation::Measured(continuity) => format!(
            "maximum root/leg/foot/knee step {:.4}/{:.4}/{:.4}/{:.4} m; rotation {:.2} deg; foot rotation {:.2} deg; worst displacement {}; worst rotation {}; supported slip {}",
            continuity.maximum_root_relative_step_metres.metres(),
            continuity.maximum_leg_root_relative_step_metres.metres(),
            continuity.maximum_foot_root_relative_step_metres.metres(),
            continuity.maximum_knee_root_relative_step_metres.metres(),
            continuity.maximum_bone_rotation_step_degrees.degrees(),
            continuity.maximum_foot_rotation_step_degrees.degrees(),
            describe(&continuity.worst_displacement, |value| format!("{:.4} m", value.metres()))?,
            describe(&continuity.worst_rotation, |value| format!("{:.2} deg", value.degrees()))?,
            describe_observation(&continuity.maximum_supported_foot_slip_metres_per_frame, |value| format!("{:.4} m/frame", value.metres())),
        ),
        Observation::NotApplicable => "not applicable".to_owned(),
        Observation::Unavailable(reason) => format!("unavailable: {}", reason.description()),
    };
    let rows = [
        ("continuity", continuity),
        ("loop seam", describe_observation(&metrics.loop_seam, |value| format!("{:.4} m; {:.2} deg", value.position_metres.metres(), value.rotation_degrees.degrees()))),
        ("plant drift", describe_observation(&metrics.maximum_planted_foot_drift_metres, |value| format!("{:.4} m", value.metres()))),
        ("guard liveness", describe_observation(&metrics.guard_step_liveness, |value| format!("{}/{} visible/completed half-steps; minimum travel {:.4} m; clearance gain {:.4} m", value.visible_half_steps, value.completed_half_steps, value.minimum_swing_travel_metres.metres(), value.minimum_swing_clearance_gain_metres.metres()))),
        ("flight sole clearance", describe_observation(&metrics.clearance.minimum_flight_sole_clearance_metres, |value| format!("{:.4} m", value.metres()))),
        ("contact sole clearance", describe_observation(&metrics.clearance.contact_sole_clearance_metres, |value| format!("{:.4} to {:.4} m", value.minimum().metres(), value.maximum().metres()))),
        ("flight toe clearance", describe_observation(&metrics.clearance.minimum_flight_toe_clearance_metres, |value| format!("{:.4} m", value.metres()))),
        ("contact toe clearance", describe_observation(&metrics.clearance.minimum_contact_toe_clearance_metres, |value| format!("{:.4} m", value.metres()))),
        ("terrain relief", describe_observation(&metrics.clearance.foot_terrain_relief_metres, |value| format!("{:.4} m", value.metres()))),
        ("minimum foot clearance", describe_observation(&metrics.clearance.minimum_foot_clearance_metres, |value| format!("{:.4} m", value.metres()))),
        ("knee forward bend", describe_observation(&metrics.pose.minimum_knee_forward_bend_metres, |value| format!("{:.4} m", value.metres()))),
        ("signed foot track", describe_observation(&metrics.pose.minimum_signed_foot_track_metres, |value| format!("{:.4} m", value.metres()))),
        ("foot separation", describe_observation(&metrics.pose.minimum_inter_foot_separation_metres, |value| format!("{:.4} m", value.metres()))),
        ("knee flexion", describe_observation(&metrics.pose.minimum_knee_flexion_degrees, |value| format!("{:.2} deg", value.degrees()))),
        ("knee hemisphere", describe_observation(&metrics.pose.minimum_knee_hemisphere_dot, |value| format!("{:.3}", value.value()))),
        ("knee/foot yaw offset", describe_observation(&metrics.pose.maximum_knee_foot_yaw_offset_degrees, |value| format!("{:.2} deg", value.degrees()))),
        ("maximum facing error", describe_observation(&metrics.facing.maximum_facing_motion_error_degrees, |value| format!("{:.2} deg", value.degrees()))),
        ("tracking excess", describe_observation(&metrics.facing.maximum_facing_tracking_excess_degrees, |value| format!("{:.2} deg", value.degrees()))),
        ("guard facing error", describe_observation(&metrics.facing.maximum_guard_facing_error_degrees, |value| format!("{:.2} deg", value.degrees()))),
        ("final facing error", describe_observation(&metrics.facing.final_facing_motion_error_degrees, |value| format!("{:.2} deg", value.degrees()))),
        ("dive axis/travel error", describe_observation(&metrics.facing.maximum_dive_axis_motion_error_degrees, |value| format!("{:.2} deg", value.degrees()))),
    ].into_iter().map(|(name, value)| format!("<tr><th>{name}</th><td>{value}</td></tr>")).collect::<String>();
    Ok(format!("<h2>{}</h2><p>{} captured frames</p><table>{rows}</table>", metrics.scenario.label(), metrics.frame_count))
}

fn height_metrics_html(manifest: &AnimationCaptureManifest) -> String {
    let hard_stop = describe_observation(
        &manifest.validation.hard_stop_maximum_pelvis_step_metres,
        |step| format!("{:.4} m", step.metres()),
    );
    let rows = manifest.scenarios.iter().map(|metrics| {
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            metrics.scenario.label(),
            describe_observation(&metrics.height.pelvis_vertical_range_metres, |value| format!("{:.4}", value.metres())),
            describe_observation(&metrics.height.maximum_pelvis_vertical_step_metres, |value| format!("{:.4}", value.metres())),
            describe_observation(&metrics.height.controller_vertical_range_metres, |value| format!("{:.4}", value.metres())),
            describe_observation(&metrics.height.phase_height_range_metres, |value| format!("{:.4}", value.metres())),
            describe_observation(&metrics.height.contact_to_passing_height_gain_metres, |value| format!("{:.4}", value.metres())),
            describe_observation(&metrics.height.visual_height_peaks, |value| format!("maximum {} peaks; every cycle has two passing peaks {}", value.maximum_count(), value.each_cycle_has_two_passing_peaks())),
            describe_observation(&metrics.height.maximum_no_support_seconds, |value| format!("{:.4}", value.seconds())),
        )
    }).collect::<String>();
    format!("<h2>Height evidence</h2><p>Hard-stop maximum pelvis step: {hard_stop}</p><table><thead><tr><th>scenario</th><th>pelvis range m</th><th>maximum pelvis step m</th><th>controller range m</th><th>phase range m</th><th>contact-to-passing gain m</th><th>height peaks</th><th>maximum unsupported seconds</th></tr></thead><tbody>{rows}</tbody></table>")
}

fn describe_observation<T>(
    observation: &Observation<T>,
    measured: impl FnOnce(&T) -> String,
) -> String {
    match observation {
        Observation::NotApplicable => "not applicable".to_owned(),
        Observation::Unavailable(reason) => format!("unavailable: {}", reason.description()),
        Observation::Measured(value) => measured(value),
    }
}
