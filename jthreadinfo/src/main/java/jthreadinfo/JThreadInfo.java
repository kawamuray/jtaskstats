package jthreadinfo;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;

import sun.jvm.hotspot.HotSpotAgent;
import sun.jvm.hotspot.oops.OopUtilities;
import sun.jvm.hotspot.runtime.JavaThread;
import sun.jvm.hotspot.runtime.Threads;
import sun.jvm.hotspot.runtime.VM;
import sun.jvm.hotspot.tools.Tool;

public class JThreadInfo extends Tool {
    private static List<ThreadInfo> collectThreadsInfo() {
        VM vm = VM.getVM();
        Threads threads = vm.getThreads();
        List<ThreadInfo> infos = new ArrayList<>();
        for (JavaThread th = threads.first(); th != null; th = th.next()) {
            long tid = 0;
            if (th.isJavaThread()) {
                tid = OopUtilities.threadOopGetTID(th.getThreadObj());
            }
            long nid = th.getOSThread().threadId();
            String name = th.getThreadName();
            ThreadInfo info = new ThreadInfo(tid, nid, name);
            infos.add(info);
        }
        return infos;
    }

    @Override
    public void run() {
        for (ThreadInfo info : collectThreadsInfo()) {
            System.out.printf("Thread %d, nid=%d - %s\n", info.tid, info.nid, info.name);
        }
    }

    private static <T> T manualAttachPid(int pid, Supplier<T> work) {
        HotSpotAgent agent = new HotSpotAgent();
        agent.attach(pid);
        try {
            return work.get();
        } finally {
            agent.detach();
        }
    }

    public static ThreadInfo[] listThreads(int pid) {
        List<ThreadInfo> infos = manualAttachPid(pid, JThreadInfo::collectThreadsInfo);
        return infos.toArray(new ThreadInfo[0]);
    }

    public static void main(String[] args) {
        JThreadInfo threadInfo = new JThreadInfo();
        threadInfo.execute(args);
    }
}
