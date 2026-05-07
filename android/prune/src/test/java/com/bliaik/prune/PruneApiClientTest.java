package com.bliaik.prune;

import org.junit.Test;

import java.io.File;
import java.io.IOException;
import java.io.OutputStream;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class PruneApiClientTest {
    @Test
    public void downloadLatestApkReportsThrottledProgressForLargeFiles() throws Exception {
        byte[] payload = new byte[2 * 1024 * 1024];
        for (int i = 0; i < payload.length; i++) {
            payload[i] = (byte) (i % 251);
        }

        AtomicInteger requestCount = new AtomicInteger();
        try (FixedBodyServer server = new FixedBodyServer(payload, requestCount)) {
            server.start();
            File target = Files.createTempFile("prune-apk", ".apk").toFile();
            List<Long> progress = new ArrayList<>();
            PruneApiClient client = new PruneApiClient(
                    "http://127.0.0.1:" + server.port(),
                    "token"
            );

            client.downloadLatestApk(target, (currentBytes, totalBytes) ->
                    progress.add(currentBytes)
            );

            assertEquals(1, requestCount.get());
            assertTrue("expected throttled progress callbacks, got " + progress.size(), progress.size() <= 20);
            assertEquals(payload.length, target.length());
            assertTrue(progress.get(0) == 0L);
            assertEquals((Long) (long) payload.length, progress.get(progress.size() - 1));
        }
    }

    private static final class FixedBodyServer implements AutoCloseable {
        private final byte[] payload;
        private final AtomicInteger requestCount;
        private final ServerSocket serverSocket;

        FixedBodyServer(byte[] payload, AtomicInteger requestCount) throws IOException {
            this.payload = payload;
            this.requestCount = requestCount;
            this.serverSocket = new ServerSocket(0);
        }

        int port() {
            return serverSocket.getLocalPort();
        }

        void start() {
            Thread thread = new Thread(this::serveOneRequest);
            thread.setDaemon(true);
            thread.start();
        }

        private void serveOneRequest() {
            try (Socket socket = serverSocket.accept()) {
                requestCount.incrementAndGet();
                drainHeaders(socket);
                OutputStream output = socket.getOutputStream();
                output.write(("HTTP/1.1 200 OK\r\n"
                        + "Content-Type: application/vnd.android.package-archive\r\n"
                        + "Content-Length: " + payload.length + "\r\n"
                        + "\r\n").getBytes(java.nio.charset.StandardCharsets.US_ASCII));
                output.write(payload);
                output.flush();
            } catch (IOException ignored) {
            }
        }

        private void drainHeaders(Socket socket) throws IOException {
            int previous = -1;
            int matched = 0;
            while (true) {
                int current = socket.getInputStream().read();
                if (current == -1) {
                    return;
                }
                if ((previous == '\r' && current == '\n') || (previous == '\n' && current == '\n')) {
                    matched++;
                    if (matched >= 2) {
                        return;
                    }
                } else if (current != '\r') {
                    matched = 0;
                }
                previous = current;
            }
        }

        @Override
        public void close() throws IOException {
            serverSocket.close();
        }
    }
}
