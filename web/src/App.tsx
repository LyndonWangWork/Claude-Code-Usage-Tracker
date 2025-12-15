import { useState, useCallback } from "react";
import { OverallUsageView } from "./components/OverallUsageView";
import { ProjectUsageView } from "./components/ProjectUsageView";
import { CompactOverallView } from "./components/CompactOverallView";
import { CompactProjectView } from "./components/CompactProjectView";
import { TitleBar } from "./components/TitleBar";
import { usePushBasedUsageStats } from "./hooks/useUsage";
import React from "react";

// Window size constants
const NORMAL_SIZE = { width: 1280, height: 720 };
const COMPACT_SIZE = { width: 200, height: 147 };

export default function App() {
  const [activeView, setActiveView] = useState<"overall" | "projects">(
    "overall"
  );
  const [isCompact, setIsCompact] = useState(false);

  // Toggle compact mode with window resize
  const toggleCompact = useCallback(async (compact: boolean) => {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const { LogicalSize } = await import("@tauri-apps/api/dpi");
      const appWindow = getCurrentWindow();

      if (compact) {
        // Switch to compact mode
        await appWindow.setResizable(false);
        await appWindow.setSize(new LogicalSize(COMPACT_SIZE.width, COMPACT_SIZE.height));
      } else {
        // Switch to normal mode
        await appWindow.setSize(new LogicalSize(NORMAL_SIZE.width, NORMAL_SIZE.height));
        await appWindow.setResizable(true);
        await appWindow.center();
      }
    } catch (e) {
      // Not in Tauri environment, ignore
    }
    setIsCompact(compact);
  }, []);

  // Shared data state with push-based updates from backend
  // - Initial load and manual refresh: full fetch with loading state
  // - Updates: backend pushes changes via Tauri events (no polling)
  const { data, loading, error, fullRefetch, isAnimating } = usePushBasedUsageStats();

  if (isCompact) {
    return (
      <div className="w-[200px] h-[147px] bg-[#1e1e1e] text-gray-100 overflow-hidden">
        {/* Compact Title Bar with View Toggle */}
        <TitleBar
          onRefresh={fullRefetch}
          onToggleCompact={() => toggleCompact(false)}
          isRefreshing={loading}
          isCompact={true}
          activeView={activeView}
          onToggleView={() => setActiveView(activeView === "overall" ? "projects" : "overall")}
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
        onToggleCompact={() => toggleCompact(true)}
        isRefreshing={loading}
        isCompact={false}
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
