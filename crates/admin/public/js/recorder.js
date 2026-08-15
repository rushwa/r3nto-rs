// MediaRecorder bridge with AUTOMATIC WATERMARKING for Rento virtual tours
// Exposed via window.RentoRecorder for wasm-bindgen calls

(function() {
    'use strict';

    // Internal state
    let mediaStream = null;
    let mediaRecorder = null;
    let recordedChunks = [];
    let recordingStartTime = null;
    let currentVideoElement = null;

    // Watermark state
    let watermarkCanvas = null;
    let watermarkCtx = null;
    let canvasStream = null;
    let animationFrameId = null;
    let watermarkConfig = {
        agentId: null,
        propertyTitle: null,
        logoText: 'R3NTO',
        showTimestamp: true,
    };

    window.RentoRecorder = {
        // ═══════════════════════════════════════════
        // WATERMARK CONFIGURATION
        // ═══════════════════════════════════════════
        setWatermark: function(config) {
            // config comes as JSON string from Rust
            let parsed = config;
            if (typeof config === 'string') {
                try {
                    parsed = JSON.parse(config);
                } catch(e) {
                    console.error('Failed to parse watermark config:', e);
                    parsed = {};
                }
            }

            watermarkConfig = {
                agentId: parsed.agentId || null,
                propertyTitle: parsed.propertyTitle || null,
                logoText: parsed.logoText || 'R3NTO',
                showTimestamp: parsed.showTimestamp !== false,
            };
            console.log('🎨 Watermark configured:', watermarkConfig);
        },

        // ═══════════════════════════════════════════
        // CAMERA CONTROL
        // ═══════════════════════════════════════════
        startCamera: async function(videoElementId, facingMode) {
            try {
                const videoElement = document.getElementById(videoElementId);
                if (!videoElement) {
                    console.error('Video element not found:', videoElementId);
                    return false;
                }
                currentVideoElement = videoElement;

                // Stop any existing stream
                if (mediaStream) {
                    mediaStream.getTracks().forEach(track => track.stop());
                    mediaStream = null;
                }

                // Request camera access
                const constraints = {
                    video: {
                        facingMode: facingMode || 'environment',
                        width: { ideal: 1280 },
                        height: { ideal: 720 }
                    },
                    audio: true
                };

                mediaStream = await navigator.mediaDevices.getUserMedia(constraints);
                videoElement.srcObject = mediaStream;
                videoElement.muted = true;
                videoElement.playsInline = true;
                await videoElement.play();

                console.log('✅ Camera started successfully');
                return true;
            } catch (err) {
                console.error('❌ Camera error:', err);
                return false;
            }
        },

        stopCamera: function() {
            if (mediaStream) {
                mediaStream.getTracks().forEach(track => track.stop());
                mediaStream = null;
            }
            if (currentVideoElement) {
                currentVideoElement.srcObject = null;
            }
            currentVideoElement = null;
        },

        switchCamera: async function(facingMode) {
            const videoId = currentVideoElement ? currentVideoElement.id : 'tour-video';
            return await this.startCamera(videoId, facingMode);
        },

        // ═══════════════════════════════════════════
        // WATERMARK DRAWING
        // ═══════════════════════════════════════════
        drawWatermark: function(ctx, canvas, timestamp) {
            const w = canvas.width;
            const h = canvas.height;

            // Semi-transparent dark bar at top
            ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
            ctx.fillRect(0, 0, w, 60);

            // Semi-transparent dark bar at bottom
            ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
            ctx.fillRect(0, h - 50, w, 50);

            // R3NTO Logo (top-left)
            ctx.fillStyle = '#FFD700';  // Gold
            ctx.font = 'bold 28px Arial, sans-serif';
            ctx.textBaseline = 'middle';
            ctx.textAlign = 'left';
            ctx.fillText(watermarkConfig.logoText, 20, 30);

            // Agent ID (top-right)
            if (watermarkConfig.agentId) {
                ctx.fillStyle = '#FFFFFF';
                ctx.font = 'bold 18px Arial, sans-serif';
                ctx.textAlign = 'right';
                const agentText = `Agent: ${watermarkConfig.agentId.substring(0, 8)}...`;
                ctx.fillText(agentText, w - 20, 30);
            }

            // Timestamp (bottom-left)
            if (watermarkConfig.showTimestamp) {
                ctx.fillStyle = '#FFFFFF';
                ctx.font = '16px monospace';
                ctx.textAlign = 'left';
                ctx.textBaseline = 'middle';
                const timeStr = timestamp.toLocaleString('en-GB', {
                    year: 'numeric',
                    month: '2-digit',
                    day: '2-digit',
                    hour: '2-digit',
                    minute: '2-digit',
                    second: '2-digit',
                    hour12: false
                });
                ctx.fillText(`📅 ${timeStr}`, 20, h - 25);
            }

            // Property title (bottom-right)
            if (watermarkConfig.propertyTitle) {
                ctx.fillStyle = '#FFFFFF';
                ctx.font = '16px Arial, sans-serif';
                ctx.textAlign = 'right';
                ctx.textBaseline = 'middle';
                // Truncate long titles
                const title = watermarkConfig.propertyTitle.length > 30
                    ? watermarkConfig.propertyTitle.substring(0, 30) + '...'
                    : watermarkConfig.propertyTitle;
                ctx.fillText(`🏠 ${title}`, w - 20, h - 25);
            }
        },

        startWatermarkLoop: function() {
            if (!currentVideoElement || !mediaStream) return;

            // Get video dimensions
            const videoWidth = currentVideoElement.videoWidth || 1280;
            const videoHeight = currentVideoElement.videoHeight || 720;

            // Create offscreen canvas
            watermarkCanvas = document.createElement('canvas');
            watermarkCanvas.width = videoWidth;
            watermarkCanvas.height = videoHeight;
            watermarkCtx = watermarkCanvas.getContext('2d');

            // Capture stream from canvas at 30fps
            canvasStream = watermarkCanvas.captureStream(30);

            // Add audio track from original stream
            const audioTracks = mediaStream.getAudioTracks();
            if (audioTracks.length > 0) {
                canvasStream.addTrack(audioTracks[0]);
            }

            // Animation loop: draw video frame + watermark
            const self = this;
            function renderFrame() {
                if (!currentVideoElement || currentVideoElement.paused || currentVideoElement.ended) {
                    animationFrameId = requestAnimationFrame(renderFrame);
                    return;
                }

                // Draw video frame
                watermarkCtx.drawImage(currentVideoElement, 0, 0, videoWidth, videoHeight);

                // Draw watermark overlay
                self.drawWatermark(watermarkCtx, watermarkCanvas, new Date());

                animationFrameId = requestAnimationFrame(renderFrame);
            }

            renderFrame();
            console.log('🎨 Watermark loop started');
        },

        stopWatermarkLoop: function() {
            if (animationFrameId) {
                cancelAnimationFrame(animationFrameId);
                animationFrameId = null;
            }
            if (canvasStream) {
                canvasStream.getTracks().forEach(track => track.stop());
                canvasStream = null;
            }
            watermarkCanvas = null;
            watermarkCtx = null;
        },

        // ═══════════════════════════════════════════
        // RECORDING CONTROL
        // ═══════════════════════════════════════════
        startRecording: function() {
            if (!mediaStream) {
                console.error('Camera not started');
                return false;
            }

            try {
                recordedChunks = [];

                // Start the watermark rendering loop
                this.startWatermarkLoop();

                // Wait a moment for canvas stream to initialize
                setTimeout(() => {
                    // Record from CANVAS stream (with watermark) instead of raw camera
                    const streamToRecord = canvasStream || mediaStream;

                    let mimeType = 'video/webm;codecs=vp9,opus';
                    if (!MediaRecorder.isTypeSupported(mimeType)) {
                        mimeType = 'video/webm;codecs=vp8,opus';
                    }
                    if (!MediaRecorder.isTypeSupported(mimeType)) {
                        mimeType = 'video/webm';
                    }

                    mediaRecorder = new MediaRecorder(streamToRecord, {
                        mimeType: mimeType,
                        videoBitsPerSecond: 2500000  // 2.5 Mbps
                    });

                    mediaRecorder.ondataavailable = (event) => {
                        if (event.data && event.data.size > 0) {
                            recordedChunks.push(event.data);
                        }
                    };

                    mediaRecorder.onerror = (event) => {
                        console.error('MediaRecorder error:', event.error);
                    };

                    // Start collecting data every second
                    mediaRecorder.start(1000);
                    recordingStartTime = Date.now();
                    console.log('🔴 Recording started WITH watermark');
                }, 100);

                return true;
            } catch (err) {
                console.error('Recording error:', err);
                return false;
            }
        },

        stopRecording: function() {
            if (mediaRecorder && mediaRecorder.state !== 'inactive') {
                mediaRecorder.stop();
            }

            this.stopWatermarkLoop();

            const duration = recordingStartTime
                ? Math.floor((Date.now() - recordingStartTime) / 1000)
                : 0;
            recordingStartTime = null;
            console.log('⏹ Recording stopped (with watermark), duration:', duration, 's');
            return duration;
        },

        // ═══════════════════════════════════════════
        // PREVIEW & BLOB ACCESS
        // ═══════════════════════════════════════════
        getRecordedBlobUrl: function() {
            if (recordedChunks.length === 0) return null;
            const mimeType = mediaRecorder && mediaRecorder.mimeType
                ? mediaRecorder.mimeType
                : 'video/webm';
            const blob = new Blob(recordedChunks, { type: mimeType });
            return URL.createObjectURL(blob);
        },

        getRecordedSize: function() {
            return recordedChunks.reduce((sum, chunk) => sum + chunk.size, 0);
        },

        getRecordedMimeType: function() {
            return (mediaRecorder && mediaRecorder.mimeType) || 'video/webm';
        },

        // ═══════════════════════════════════════════
// FILE UPLOAD
// ═══════════════════════════════════════════
        uploadVideo: async function(tourRequestId, authToken, onProgress) {
            if (recordedChunks.length === 0) {
                throw new Error('No video recorded');
            }

            const mimeType = (mediaRecorder && mediaRecorder.mimeType) || 'video/webm';
            const blob = new Blob(recordedChunks, { type: mimeType });

            const formData = new FormData();
            formData.append('tour_request_id', tourRequestId);
            formData.append('duration_seconds', String(Math.floor((Date.now() - (recordingStartTime || Date.now())) / 1000) || 0));
            formData.append('video', blob, `tour_${tourRequestId}.webm`);

            return new Promise((resolve, reject) => {
                const xhr = new XMLHttpRequest();

                xhr.upload.onprogress = (event) => {
                    if (event.lengthComputable && onProgress) {
                        const percent = Math.round((event.loaded / event.total) * 100);
                        onProgress(percent);
                    }
                };

                xhr.onload = () => {
                    if (xhr.status >= 200 && xhr.status < 300) {
                        try {
                            const response = JSON.parse(xhr.responseText);
                            resolve(response);
                        } catch (e) {
                            reject(new Error('Invalid response from server'));
                        }
                    } else {
                        reject(new Error(`Upload failed: ${xhr.status} ${xhr.statusText}`));
                    }
                };

                xhr.onerror = () => reject(new Error('Network error during upload'));
                xhr.ontimeout = () => reject(new Error('Upload timed out'));

                xhr.open('POST', 'http://localhost:8000/api/tours/upload-video');
                xhr.setRequestHeader('Authorization', `Bearer ${authToken}`);
                xhr.timeout = 300000; // 5 minutes timeout for large videos
                xhr.send(formData);
            });
        },

        // ═══════════════════════════════════════════
        // UTILITIES
        // ═══════════════════════════════════════════
        isSupported: function() {
            return !!(
                navigator.mediaDevices &&
                navigator.mediaDevices.getUserMedia &&
                window.MediaRecorder &&
                HTMLCanvasElement.prototype.captureStream
            );
        },

        getCurrentTime: function() {
            return Date.now();
        },

        cleanup: function() {
            this.stopRecording();
            this.stopCamera();
            recordedChunks = [];
        }
    };

    // Auto-cleanup on page unload
    window.addEventListener('beforeunload', () => {
        if (window.RentoRecorder) {
            window.RentoRecorder.cleanup();
        }
    });

    console.log('✅ RentoRecorder initialized (with watermarking). Supported:', window.RentoRecorder.isSupported());
})();