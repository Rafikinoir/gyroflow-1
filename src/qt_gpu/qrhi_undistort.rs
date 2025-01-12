// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright © 2021-2022 Adrian <adrian.eddy at gmail>

use qml_video_rs::video_player::MDKPlayerWrapper;
use std::sync::Arc;
use crate::core::StabilizationManager;
use crate::core::undistortion::RGBA8;
use cpp::*;
use qmetaobject::QSize;

cpp! {{
    struct RustPtr { void *data; };
    #include "src/qt_gpu/qrhi_undistort.cpp"

    static std::unique_ptr<QtRHIUndistort> rhiUndistortion;
}}

pub fn resize_player(stab: Arc<StabilizationManager<RGBA8>>) {
    let player = cpp!(unsafe [] -> *mut MDKPlayerWrapper as "MDKPlayerWrapper *" {
        if (rhiUndistortion && !rhiUndistortion->m_pipeline.isNull() && rhiUndistortion->m_player) {
            return rhiUndistortion->m_player;
        } else {
            return nullptr;
        }
    });
    if !player.is_null() {
        unsafe { init_player(&mut *player, stab); }
    }
}
pub fn init_player(mdkplayer: &mut MDKPlayerWrapper, stab: Arc<StabilizationManager<RGBA8>>) {
    cpp!(unsafe [mdkplayer as "MDKPlayerWrapper *", stab as "RustPtr"] {
        if (!mdkplayer || !mdkplayer->mdkplayer) return;

        auto initCb = [mdkplayer, stab](QSize texSize, QSizeF itemSize) -> bool {
            rhiUndistortion = std::make_unique<QtRHIUndistort>(mdkplayer);

            QSize outputSize = rust!(Rust_Controller_InitRHI [stab: Arc<StabilizationManager<RGBA8>> as "RustPtr"] -> QSize as "QSize" {
                let osize = stab.params.read().output_size;
                QSize { width: osize.0 as u32, height: osize.1 as u32 }
            });
            return rhiUndistortion->init(mdkplayer->mdkplayer, texSize, itemSize, outputSize);
        };
        auto renderCb = [mdkplayer, stab](double timestamp, int32_t frame, bool doRender) -> bool {
            if (!rhiUndistortion) return false;

            float bg[4];
            uint32_t params_count = rust!(Rust_Controller_RenderRHIParams [stab: Arc<StabilizationManager<RGBA8>> as "RustPtr", bg: *mut f32 as "float *"] -> u32 as "uint32_t" {
                let params = stab.params.read();
                *bg.offset(0) = params.background[0] / 255.0;
                *bg.offset(1) = params.background[1] / 255.0;
                *bg.offset(2) = params.background[2] / 255.0;
                *bg.offset(3) = params.background[3] / 255.0;
                if params.frame_readout_time.abs() > 0.0 {
                    (params.size.1 + 2) as u32
                } else {
                    3
                }
            });
            if (rhiUndistortion->params_buffer.size() < params_count * 9) {
                rhiUndistortion->params_buffer.resize(params_count * 9);
            }
            auto ptr = rhiUndistortion->params_buffer.data();
            bool ok = rust!(Rust_Controller_RenderRHI [timestamp: f64 as "double", frame: i32 as "int32_t", stab: Arc<StabilizationManager<RGBA8>> as "RustPtr", ptr: *mut f32 as "float *", params_count: u32 as "uint32_t"] -> bool as "bool" {
                stab.fill_undistortion_data((timestamp * 1_000_000.0) as i64, ptr, params_count as usize * 9)
            });

            return ok && rhiUndistortion->render(mdkplayer->mdkplayer, timestamp, frame, ptr, params_count, bg, doRender, nullptr, 0, nullptr, 0);
        };

        auto cleanupCb = [] { rhiUndistortion.reset(); };

        mdkplayer->mdkplayer->cleanupGpuCompute();
        mdkplayer->mdkplayer->setupGpuCompute(initCb, renderCb, cleanupCb);
    });
}

pub fn deinit_player(mdkplayer: &mut MDKPlayerWrapper) {
    cpp!(unsafe [mdkplayer as "MDKPlayerWrapper *"] {
        if (!mdkplayer || !mdkplayer->mdkplayer) return;
        rhiUndistortion.reset();
        mdkplayer->mdkplayer->cleanupGpuCompute();
        mdkplayer->mdkplayer->setupGpuCompute(nullptr, nullptr, nullptr);
    });
}
