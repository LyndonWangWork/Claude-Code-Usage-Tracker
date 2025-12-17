import { useState, useCallback, useRef, useEffect } from "react";
import { OverallUsageView } from "./components/OverallUsageView";
import { ProjectUsageView } from "./components/ProjectUsageView";
import { CompactOverallView } from "./components/CompactOverallView";
import { CompactProjectView } from "./components/CompactProjectView";
import { MiniView } from "./components/MiniView";
import { TitleBar } from "./components/TitleBar";
import { usePushBasedUsageStats } from "./hooks/useUsage";
import React from "react";

// Window size constants
const NORMAL_SIZE = { width: 1280, height: 720 };
const COMPACT_SIZE = { width: 200, height: 147 };
const MINI_SIZE = { width: 280, height: 32 };

// View mode type
type ViewMode = "normal" | "compact" | "mini";

// Mini mode timer delay in milliseconds (10 seconds)
const MINI_MODE_DELAY = 10000;

export default function App() {
  const [activeView, setActiveView] = useState<"overall" | "projects">(
    "overall"
  );
  const [viewMode, setViewMode] = useState<ViewMode>("normal");
  const miniTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Clear mini timer helper
  const clearMiniTimer = useCallback(() => {
    if (miniTimerRef.current) {
      clearTimeout(miniTimerRef.current);
      miniTimerRef.current = null;
    }
  }, []);

  // Switch to specific view mode
  const switchToMode = useCallback(async (mode: ViewMode) => {
    clearMiniTimer();
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const { LogicalSize } = await import("@tauri-apps/api/dpi");
      const appWindow = getCurrentWindow();

      switch (mode) {
        case "normal":
          await appWindow.setSize(new LogicalSize(NORMAL_SIZE.width, NORMAL_SIZE.height));
          await appWindow.setResizable(true);
          await appWindow.center();
          break;
        case "compact":
          await appWindow.setResizable(false);
          await appWindow.setSize(new LogicalSize(COMPACT_SIZE.width, COMPACT_SIZE.height));
          break;
        case "mini":
          await appWindow.setResizable(false);
          await appWindow.setSize(new LogicalSize(MINI_SIZE.width, MINI_SIZE.height));
          break;
      }
    } catch (e) {
      // Not in Tauri environment, ignore
    }
    setViewMode(mode);
  }, [clearMiniTimer]);

  // Cycle through view modes: normal -> compact -> mini -> normal
  const cycleViewMode = useCallback(async () => {
    const nextMode: ViewMode = viewMode === "normal" ? "compact" : viewMode === "compact" ? "mini" : "normal";
    await switchToMode(nextMode);
  }, [viewMode, switchToMode]);

  // Legacy toggle functions for backward compatibility
  const toggleMini = useCallback(async (mini: boolean) => {
    await switchToMode(mini ? "mini" : "compact");
  }, [switchToMode]);

  const toggleCompact = useCallback(async (compact: boolean) => {
    await switchToMode(compact ? "compact" : "normal");
  }, [switchToMode]);

  // Start mini mode timer (only in compact mode)
  const startMiniTimer = useCallback(() => {
    if (viewMode !== "compact") return;
    clearMiniTimer();
    miniTimerRef.current = setTimeout(() => {
      toggleMini(true);
    }, MINI_MODE_DELAY);
  }, [viewMode, clearMiniTimer, toggleMini]);

  // Reset mini timer on mouse enter (keep in compact, just restart timer)
  const resetMiniTimer = useCallback(() => {
    if (viewMode !== "compact") return;
    clearMiniTimer();
    startMiniTimer();
  }, [viewMode, clearMiniTimer, startMiniTimer]);

  // Cleanup timer on unmount
  useEffect(() => {
    return () => {
      clearMiniTimer();
    };
  }, [clearMiniTimer]);

  // Shared data state with push-based updates from backend
  // - Initial load and manual refresh: full fetch with loading state
  // - Updates: backend pushes changes via Tauri events (no polling)
  const { data, loading, error, fullRefetch, isAnimating } = usePushBasedUsageStats();

  // Mini mode: single line display
  if (viewMode === "mini") {
    return (
      <MiniView
        data={data}
        loading={loading}
        error={error}
        onRestore={() => toggleMini(false)}
        onMouseEnter={resetMiniTimer}
        onCycleMode={cycleViewMode}
      />
    );
  }

  // Compact mode: small window with timer for mini mode
  if (viewMode === "compact") {
    return (
      <div
        className="w-[200px] h-[147px] bg-[#1e1e1e] text-gray-100 overflow-hidden"
        onMouseLeave={startMiniTimer}
        onMouseEnter={resetMiniTimer}
      >
        {/* Compact Title Bar with View Toggle */}
        <TitleBar
          onRefresh={fullRefetch}
          onToggleCompact={cycleViewMode}
          isRefreshing={loading}
          isCompact={true}
          activeView={activeView}
          onToggleView={() => setActiveView(activeView === "overall" ? "projects" : "overall")}
          dataSource={data?.dataSource}
        />

        {/* Compact Content - Full height without tab bar */}
        <div className="h-[calc(100%-24px)] overflow-auto">
          {activeView === "overall" ? (
            <CompactOverallView data={data} loading={loading} error={error} isAnimating={isAnimating} />
          ) : (
            <CompactProjectView data={data} loading={loading} error={error} isAnimating={isAnimating} />
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen bg-[#1e1e1e] text-gray-100 flex flex-col overflow-hidden">
      {/* Title Bar */}
      <TitleBar
        onRefresh={fullRefetch}
        onToggleCompact={cycleViewMode}
        isRefreshing={loading}
        isCompact={false}
        dataSource={data?.dataSource}
      />

      {/* Main Content */}
      <div className="flex-1 overflow-auto">
        <div className="container mx-auto px-4 py-6 max-w-6xl">
          {/* Header */}
          <div className="mb-6">
            <h1 className="text-center tracking-widest text-blue-300 mb-6">
              CLAUDE CODE USAGE TRACKER
            </h1>

            {/* View Tabs */}
            <div className="flex justify-center gap-4">
              <button
                onClick={() => setActiveView("overall")}
                className={`px-6 py-2 rounded-lg transition-colors ${
                  activeView === "overall"
                    ? "bg-blue-600 text-white"
                    : "bg-gray-700 text-gray-300 hover:bg-gray-600"
                }`}
              >
                Overall Usage
              </button>
              <button
                onClick={() => setActiveView("projects")}
                className={`px-6 py-2 rounded-lg transition-colors ${
                  activeView === "projects"
                    ? "bg-blue-600 text-white"
                    : "bg-gray-700 text-gray-300 hover:bg-gray-600"
                }`}
                title="Project Usage"
              >
                Project Usage
              </button>
            </div>
          </div>

          {/* Content */}
          {activeView === "overall" ? (
            <OverallUsageView data={data} loading={loading} error={error} refetch={fullRefetch} isAnimating={isAnimating} />
          ) : (
            <ProjectUsageView data={data} loading={loading} error={error} refetch={fullRefetch} isAnimating={isAnimating} />
          )}
        </div>
      </div>
    </div>
  );
}
