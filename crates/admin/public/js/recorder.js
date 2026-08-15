// // static/js/recorder.js
// // MediaRecorder bridge for Rento virtual tour recorder
// // Exposed via window.RentoRecorder for wasm-bindgen calls
//
// (function() {
//     'use strict';
//
//     // Internal state
//     let mediaStream = null;
//     let mediaRecorder = null;
//     let recordedChunks = [];
//     let recordingStartTime = null;
//     let currentVideoElement = null;
//
//     window.RentoRecorder = {
//         // ═══════════════════════════════════════════
//         // CAMERA CONTROL
//         // ═══════════════════════════════════════════
//         async startCamera(videoElementId, facingMode) {
//             try {
//                 const videoElement = document.getElementById(videoElementId);
//                 if (!videoElement) {
//                     console.error('Video element not found:', videoElementId);
//                     return false;
//                 }
//                 currentVideoElement = videoElement;
//
//                 // Stop any existing stream
//                 if (mediaStream) {
//                     mediaStream.getTracks().forEach(track => track.stop());
//                     mediaStream = null;
//                 }
//
//                 // Request camera access
//                 const constraints = {
//                     video: {
//                         facingMode: facingMode || 'environment',
//                         width: { ideal: 1280 },
//                         height: { ideal: 720 }
//                     },
//                     audio: true
//                 };
//
//                 mediaStream = await navigator.mediaDevices.getUserMedia(constraints);
//                 videoElement.srcObject = mediaStream;
//                 videoElement.muted = true;
//                 videoElement.playsInline = true;
//                 await videoElement.play();
//
//                 console.log('✅ Camera started successfully');
//                 return true;
//             } catch (err) {
//                 console.error('❌ Camera error:', err);
//                 return false;
//             }
//         },
//
//         stopCamera() {
//             if (mediaStream) {
//                 mediaStream.getTracks().forEach(track => track.stop());
//                 mediaStream = null;
//             }
//             if (currentVideoElement) {
//                 currentVideoElement.srcObject = null;
//             }
//             currentVideoElement = null;
//         },
//
//         async switchCamera(facingMode) {
//             const videoId = currentVideoElement ? currentVideoElement.id : 'tour-video';
//             return await this.startCamera(videoId, facingMode);
//         },
//
//         // ═══════════════════════════════════════════
//         // RECORDING CONTROL
//         // ═══════════════════════════════════════════
//         startRecording() {
//             if (!mediaStream) {
//                 console.error('Camera not started');
//                 return false;
//             }
//
//             try {
//                 recordedChunks = [];
//
//                 // Choose best supported MIME type
//                 let mimeType = 'video/webm;codecs=vp9,opus';
//                 if (!MediaRecorder.isTypeSupported(mimeType)) {
//                     mimeType = 'video/webm;codecs=vp8,opus';
//                 }
//                 if (!MediaRecorder.isTypeSupported(mimeType)) {
//                     mimeType = 'video/webm';
//                 }
//
//                 mediaRecorder = new MediaRecorder(mediaStream, {
//                     mimeType: mimeType,
//                     videoBitsPerSecond: 2500000  // 2.5 Mbps
//                 });
//
//                 mediaRecorder.ondataavailable = (event) => {
//                     if (event.data && event.data.size > 0) {
//                         recordedChunks.push(event.data);
//                     }
//                 };
//
//                 mediaRecorder.onerror = (event) => {
//                     console.error('MediaRecorder error:', event.error);
//                 };
//
//                 // Start collecting data every second
//                 mediaRecorder.start(1000);
//                 recordingStartTime = Date.now();
//
//                 console.log('🔴 Recording started');
//                 return true;
//             } catch (err) {
//                 console.error('Recording error:', err);
//                 return false;
//             }
//         },
//
//         stopRecording() {
//             if (mediaRecorder && mediaRecorder.state !== 'inactive') {
//                 mediaRecorder.stop();
//             }
//             const duration = recordingStartTime
//                 ? Math.floor((Date.now() - recordingStartTime) / 1000)
//                 : 0;
//             recordingStartTime = null;
//             console.log('⏹ Recording stopped, duration:', duration, 's');
//             return duration;
//         },
//
//         // ═══════════════════════════════════════════
//         // PREVIEW & BLOB ACCESS
//         // ═══════════════════════════════════════════
//         getRecordedBlobUrl() {
//             if (recordedChunks.length === 0) return null;
//             const mimeType = mediaRecorder && mediaRecorder.mimeType
//                 ? mediaRecorder.mimeType
//                 : 'video/webm';
//             const blob = new Blob(recordedChunks, { type: mimeType });
//             return URL.createObjectURL(blob);
//         },
//
//         getRecordedSize() {
//             return recordedChunks.reduce((sum, chunk) => sum + chunk.size, 0);
//         },
//
//         getRecordedMimeType() {
//             return (mediaRecorder && mediaRecorder.mimeType) || 'video/webm';
//         },
//
//         // ═══════════════════════════════════════════
//         // UTILITIES
//         // ═══════════════════════════════════════════
//         isSupported() {
//             return !!(
//                 navigator.mediaDevices &&
//                 navigator.mediaDevices.getUserMedia &&
//                 window.MediaRecorder
//             );
//         },
//
//         getCurrentTime() {
//             return Date.now();
//         },
//
//         cleanup() {
//             this.stopRecording();
//             this.stopCamera();
//             recordedChunks = [];
//         }
//     };
//
//     // Auto-cleanup on page unload
//     window.addEventListener('beforeunload', () => {
//         if (window.RentoRecorder) {
//             window.RentoRecorder.cleanup();
//         }
//     });
//
//     console.log('✅ RentoRecorder initialized. Supported:', window.RentoRecorder.isSupported());
// })();