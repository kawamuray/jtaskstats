package jthreadinfo;

public class ThreadInfo {
    public final long tid;
    public final long nid;
    public final String name;

    public ThreadInfo(long tid, long nid, String name) {
        this.tid = tid;
        this.nid = nid;
        this.name = name;
    }

    public boolean isJavaThread() {
        return tid != 0;
    }
}
