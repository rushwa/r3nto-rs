// Generates a stable device fingerprint for tour viewing device-lock
(function() {
    'use strict';

    // Simple hash function (djb2)
    function hashString(str) {
        let hash = 5381;
        for (let i = 0; i < str.length; i++) {
            hash = ((hash << 5) + hash) + str.charCodeAt(i);
            hash = hash & hash;  // Convert to 32-bit integer
        }
        return Math.abs(hash).toString(36);
    }

    // Generate canvas fingerprint
    function getCanvasFingerprint() {
        try {
            const canvas = document.createElement('canvas');
            canvas.width = 200;
            canvas.height = 50;
            const ctx = canvas.getContext('2d');
            ctx.textBaseline = 'top';
            ctx.font = '14px Arial';
            ctx.fillStyle = '#f60';
            ctx.fillRect(0, 0, 100, 30);
            ctx.fillStyle = '#069';
            ctx.fillText('Rento FP 🔒', 2, 15);
            return canvas.toDataURL();
        } catch (e) {
            return 'canvas-unavailable';
        }
    }

    // Generate WebGL fingerprint
    function getWebGLFingerprint() {
        try {
            const canvas = document.createElement('canvas');
            const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
            if (!gl) return 'webgl-unavailable';
            const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
            if (debugInfo) {
                return gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) +
                    '~' + gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL);
            }
            return 'webgl-no-debug';
        } catch (e) {
            return 'webgl-error';
        }
    }

    // Main fingerprint generator
    window.RentoFingerprint = {
        generate: function() {
            const components = [
                navigator.userAgent || '',
                navigator.language || '',
                screen.width + 'x' + screen.height,
                screen.colorDepth || '',
                new Date().getTimezoneOffset(),
                navigator.hardwareConcurrency || '',
                navigator.platform || '',
                getCanvasFingerprint(),
                getWebGLFingerprint(),
            ];

            const raw = components.join('|');
            // Combine multiple hashes for stability
            const fp = 'fp_' + hashString(raw) + '_' + hashString(raw.split('').reverse().join(''));

            // Store in localStorage for consistency
            try {
                localStorage.setItem('rento_device_fp', fp);
            } catch (e) {}

            return fp;
        },

        get: function() {
            // Return stored fingerprint if available (for consistency)
            try {
                const stored = localStorage.getItem('rento_device_fp');
                if (stored) return stored;
            } catch (e) {}
            return this.generate();
        }
    };

    console.log('✅ RentoFingerprint initialized');
})();