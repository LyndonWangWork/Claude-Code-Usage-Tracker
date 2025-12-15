import { DollarSign, Zap, Loader2 } from "lucide-react";
import { UsageData, formatTokens, formatCost } from "../hooks/useUsage";
import React from "react";
import { AnimatedNumber } from "./AnimatedNumber";
import { AnimatedProgressBar } from "./AnimatedProgressBar";

interface CompactProjectViewProps {
  data: UsageData | null;
  loading: boolean;
  error: string | null;
  isAnimating?: boolean;
}

export function CompactProjectView({ data, loading, error, isAnimating }: CompactProjectViewProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-4 h-4 text-blue-400 animate-spin" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="p-2 text-[10px] text-red-400 text-center">
        Error loading data
      </div>
    );
  }

  // Show top 3 projects by most recent activity
  const topProjects = [...data.projects]
    .sort((a, b) => {
      const aTime = a.lastActivity || '';
      const bTime = b.lastActivity || '';
      return bTime.localeCompare(aTime); // Most recent first
    })
    .slice(0, 3);
  const { overallStats } = data;
  const totalCost = overallStats.totalCostUsd || 1;
  const totalTokens = (overallStats.totalInputTokens + overallStats.totalOutputTokens) || 1;

  if (topProjects.length === 0) {
    return (
      <div className="p-2 text-[10px] text-gray-500 text-center">
        No projects found
      </div>
    );
  }

  return (
    <div className="p-2 space-y-1">
      {topProjects.map((project) => {
        const projectTokens =
          project.totalInputTokens + project.totalOutputTokens;
        return (
          <div
            key={project.projectPath}
            className="bg-gray-800/30 rounded p-1 border border-gray-700/50"
          >
            <div
              className="text-[10px] text-gray-200 mb-0.5 truncate"
              title={project.projectPath}
            >
              {project.displayName}
            </div>

            {/* Cost */}
            <div className="mb-0.5">
              <div className="flex items-center justify-between mb-0.5">
                <div className="flex items-center gap-0.5">
                  <DollarSign className="w-2.5 h-2.5 text-green-400" />
                  <span className="text-[8px] text-gray-400">Cost</span>
                </div>
                <span className="text-[9px] text-gray-200">
                  <AnimatedNumber value={project.totalCostUsd} formatter={formatCost} duration={300} />
                </span>
              </div>
              <AnimatedProgressBar
                value={project.totalCostUsd}
                maxValue={totalCost}
                color="green"
                isAnimating={isAnimating}
              />
            </div>

            {/* Tokens */}
            <div>
              <div className="flex items-center justify-between mb-0.5">
                <div className="flex items-center gap-0.5">
                  <Zap className="w-2.5 h-2.5 text-yellow-400" />
                  <span className="text-[8px] text-gray-400">Tokens</span>
                </div>
                <span className="text-[9px] text-gray-200">
                  <AnimatedNumber value={projectTokens} formatter={formatTokens} duration={300} />
                </span>
              </div>
              <AnimatedProgressBar
                value={projectTokens}
                maxValue={totalTokens}
                color="yellow"
                isAnimating={isAnimating}
              />
            </div>
          </div>
        );
      })}

      <div className="text-center text-[8px] text-gray-500 pt-0.5">
        Top {topProjects.length} of {data.projects.length} projects
      </div>
    </div>
  );
}
