package jthreadinfo;

public final class SampleMain {
    public static void main(String[] args) {
        start("Thread-A");
        start("Thread-B");
        start("Thread-C");
    }

    private static void start(String name) {
        Thread th = new Thread(() -> {
            try {
                Thread.sleep(10000);
            } catch (InterruptedException e) {
                throw new RuntimeException(e);
            }
        });
        th.setName(name);
        th.start();
    }
}
