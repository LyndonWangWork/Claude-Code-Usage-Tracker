import {
  DollarSign,
  Zap,
  MessageSquare,
  ChevronDown,
  ChevronRight,
  FolderOpen,
  Folder,
  AlertCircle,
  Loader2,
  Search,
  RefreshCw,
} from "lucide-react";
import { useState, useMemo } from "react";
import React from "react";
import {
  UsageData,
  ProjectStats,
  formatTokens,
  formatCost,
  formatRelativeTime,
} from "../hooks/useUsage";
import { AnimatedNumber } from "./AnimatedNumber";
import { AnimatedProgressBar } from "./AnimatedProgressBar";

interface ProjectUsageViewProps {
  data: UsageData | null;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  isAnimating?: boolean;
}

export function ProjectUsageView({ data, loading, error, refetch, isAnimating }: ProjectUsageViewProps) {
  const [expandedProjects, setExpandedProjects] = useState<Set<string>>(
    new Set()
  );
  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<
    "cost" | "tokens" | "messages" | "activity"
  >("activity");
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");

  // Filter and sort projects
  const filteredProjects = useMemo(() => {
    if (!data?.projects) return [];

    let result = data.projects.filter(
      (p) =>
        p.displayName.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.projectPath.toLowerCase().includes(searchQuery.toLowerCase())
    );

    result.sort((a, b) => {
      let cmp = 0;
      switch (sortBy) {
        case "cost":
          cmp = a.totalCostUsd - b.totalCostUsd;
          break;
        case "tokens":
          cmp =
            a.totalInputTokens +
            a.totalOutputTokens -
            (b.totalInputTokens + b.totalOutputTokens);
          break;
        case "messages":
          cmp = a.messageCount - b.messageCount;
          break;
        case "activity":
          const aTime = a.lastActivity || "";
          const bTime = b.lastActivity || "";
          cmp = aTime.localeCompare(bTime);
          break;
      }
      return sortOrder === "desc" ? -cmp : cmp;
    });

    return result;
  }, [data?.projects, searchQuery, sortBy, sortOrder]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="w-8 h-8 text-blue-400 animate-spin" />
        <span className="ml-3 text-gray-400">Loading projects...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="border border-red-700 rounded-lg p-6 bg-red-900/20">
        <div className="flex items-center gap-3 mb-4">
          <AlertCircle className="w-6 h-6 text-red-400" />
          <h3 className="text-red-400">Error loading projects</h3>
        </div>
        <p className="text-gray-400 mb-4">{error}</p>
        <button
          onClick={refetch}
          className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-gray-300"
        >
          <RefreshCw className="w-4 h-4" />
          Retry
        </button>
      </div>
    );
  }

  if (!data || data.projects.length === 0) {
    return (
      <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50 text-center">
        <Folder className="w-12 h-12 text-gray-600 mx-auto mb-4" />
        <p className="text-gray-400">No projects found</p>
        <p className="text-gray-500 text-sm mt-2">
          Start using Claude Code in your projects to see usage statistics here.
        </p>
      </div>
    );
  }

  const { overallStats } = data;
  const totalTokens =
    overallStats.totalInputTokens + overallStats.totalOutputTokens;

  const toggleProject = (projectPath: string) => {
    const newExpanded = new Set(expandedProjects);
    if (newExpanded.has(projectPath)) {
      newExpanded.delete(projectPath);
    } else {
      newExpanded.add(projectPath);
    }
    setExpandedProjects(newExpanded);
  };

  const toggleSort = (field: typeof sortBy) => {
    if (sortBy === field) {
      setSortOrder(sortOrder === "asc" ? "desc" : "asc");
    } else {
      setSortBy(field);
      setSortOrder("desc");
    }
  };

  const ProjectRow = ({ project }: { project: ProjectStats; key: string }) => {
    const isExpanded = expandedProjects.has(project.projectPath);
    const projectTokens = project.totalInputTokens + project.totalOutputTokens;
    const costPercent =
      totalTokens > 0
        ? (project.totalCostUsd / overallStats.totalCostUsd) * 100
        : 0;
    const tokenPercent =
      totalTokens > 0 ? (projectTokens / totalTokens) * 100 : 0;
    const messagePercent =
      overallStats.totalMessages > 0
        ? (project.messageCount / overallStats.totalMessages) * 100
        : 0;

    return (
      <div className="border border-gray-700 rounded-lg mb-4 bg-gray-900/50 overflow-hidden">
        {/* Project Header */}
        <button
          onClick={() => toggleProject(project.projectPath)}
          className="w-full p-4 flex items-center justify-between hover:bg-gray-800/50 transition-colors"
        >
          <div className="flex items-center gap-3 min-w-0 flex-1 cursor-pointer">
            {isExpanded ? (
              <ChevronDown className="w-5 h-5 text-gray-400 flex-shrink-0" />
            ) : (
              <ChevronRight className="w-5 h-5 text-gray-400 flex-shrink-0" />
            )}
            {isExpanded ? (
              <FolderOpen className="w-5 h-5 text-blue-400 flex-shrink-0" />
            ) : (
              <Folder className="w-5 h-5 text-blue-400 flex-shrink-0" />
            )}
            <span
              className="text-gray-200 truncate"
              title={project.projectPath}
            >
              {project.displayName}
            </span>
            <span className="text-gray-500 text-sm flex-shrink-0">
              • {formatRelativeTime(project.lastActivity)}
            </span>
          </div>
          <div className="flex items-center flex-shrink-0">
            <div className="flex items-center gap-1 w-[140px] justify-end">
              <DollarSign className="w-4 h-4 text-green-400" />
              <span className="text-gray-300 w-[60px] text-left">
                <AnimatedNumber value={project.totalCostUsd} formatter={formatCost} />
              </span>
              <span className="text-gray-500 text-sm w-[50px] text-right">
                ({costPercent.toFixed(1)}%)
              </span>
            </div>
            <div className="flex items-center gap-1 w-[90px] justify-end">
              <Zap className="w-4 h-4 text-yellow-400" />
              <span className="text-gray-300 w-[55px] text-left">
                <AnimatedNumber value={projectTokens} formatter={formatTokens} />
              </span>
            </div>
            <div className="flex items-center gap-1 w-[80px] justify-end">
              <MessageSquare className="w-4 h-4 text-blue-400" />
              <span className="text-gray-300 w-[50px] text-left">
                <AnimatedNumber value={project.messageCount} formatter={(v) => Math.round(v).toString()} />
              </span>
            </div>
          </div>
        </button>

        {/* Project Details (Expanded) */}
        {isExpanded && (
          <div className="px-4 pb-4 space-y-4 bg-gray-800/30">
            {/* Full Path */}
            <div
              className="text-xs text-gray-500 truncate"
              title={project.projectPath}
            >
              {project.projectPath}
            </div>

            {/* Cost Breakdown */}
            <div>
              <div className="flex items-center gap-2 mb-2 text-sm">
                <DollarSign className="w-4 h-4 text-green-400" />
                <span className="text-gray-400">Cost Usage</span>
                <span className="text-gray-500 ml-auto">
                  {costPercent.toFixed(1)}% of total
                </span>
              </div>
              <AnimatedProgressBar
                value={project.totalCostUsd}
                maxValue={overallStats.totalCostUsd}
                color="green"
                height="h-2"
                isAnimating={isAnimating}
              />
            </div>

            {/* Token Breakdown */}
            <div>
              <div className="flex items-center gap-2 mb-2 text-sm">
                <Zap className="w-4 h-4 text-yellow-400" />
                <span className="text-gray-400">Token Usage</span>
                <span className="text-gray-500 ml-auto">
                  {tokenPercent.toFixed(1)}% of total
                </span>
              </div>
              <AnimatedProgressBar
                value={projectTokens}
                maxValue={totalTokens}
                color="yellow"
                height="h-2"
                isAnimating={isAnimating}
              />
            </div>

            {/* Message Breakdown */}
            <div>
              <div className="flex items-center gap-2 mb-2 text-sm">
                <MessageSquare className="w-4 h-4 text-blue-400" />
                <span className="text-gray-400">Message Usage</span>
                <span className="text-gray-500 ml-auto">
                  {messagePercent.toFixed(1)}% of total
                </span>
              </div>
              <AnimatedProgressBar
                value={project.messageCount}
                maxValue={overallStats.totalMessages}
                color="blue"
                height="h-2"
                isAnimating={isAnimating}
              />
            </div>

            {/* Additional Stats */}
            <div className="grid grid-cols-4 gap-4 pt-2 text-sm">
              <div className="text-center p-2 bg-gray-900/50 rounded">
                <div className="text-gray-500">Input Tokens</div>
                <div className="text-gray-300">
                  <AnimatedNumber value={project.totalInputTokens} formatter={formatTokens} />
                </div>
              </div>
              <div className="text-center p-2 bg-gray-900/50 rounded">
                <div className="text-gray-500">Output Tokens</div>
                <div className="text-gray-300">
                  <AnimatedNumber value={project.totalOutputTokens} formatter={formatTokens} />
                </div>
              </div>
              <div className="text-center p-2 bg-gray-900/50 rounded">
                <div className="text-gray-500">Sessions</div>
                <div className="text-gray-300">
                  <AnimatedNumber value={project.sessionCount} formatter={(v) => Math.round(v).toString()} />
                </div>
              </div>
              <div className="text-center p-2 bg-gray-900/50 rounded">
                <div className="text-gray-500">Avg Cost/Msg</div>
                <div className="text-gray-300">
                  <AnimatedNumber
                    value={project.messageCount > 0 ? project.totalCostUsd / project.messageCount : 0}
                    formatter={formatCost}
                  />
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="space-y-6">
      {/* Summary Cards */}
      <div className="grid  sm:grid-cols-3 gap-6">
        <div className="border border-gray-700 rounded-lg p-6 bg-gradient-to-br from-green-900/20 to-gray-900/50">
          <div className="flex items-center gap-3 mb-2">
            <DollarSign className="w-6 h-6 text-green-400" />
            <h3 className="text-gray-400">Total Cost</h3>
          </div>
          <p className="text-3xl text-gray-100">
            <AnimatedNumber value={overallStats.totalCostUsd} formatter={formatCost} />
          </p>
          <p className="text-sm text-gray-500 mt-1">
            Across {overallStats.projectCount} projects
          </p>
        </div>

        <div className="border border-gray-700 rounded-lg p-6 bg-gradient-to-br from-yellow-900/20 to-gray-900/50">
          <div className="flex items-center gap-3 mb-2">
            <Zap className="w-6 h-6 text-yellow-400" />
            <h3 className="text-gray-400">Total Tokens</h3>
          </div>
          <p className="text-3xl text-gray-100">
            <AnimatedNumber value={totalTokens} formatter={formatTokens} />
          </p>
          <p className="text-sm text-gray-500 mt-1">Processed tokens</p>
        </div>

        <div className="border border-gray-700 rounded-lg p-6 bg-gradient-to-br from-blue-900/20 to-gray-900/50">
          <div className="flex items-center gap-3 mb-2">
            <MessageSquare className="w-6 h-6 text-blue-400" />
            <h3 className="text-gray-400">Total Messages</h3>
          </div>
          <p className="text-3xl text-gray-100">
            <AnimatedNumber value={overallStats.totalMessages} formatter={(v) => Math.round(v).toLocaleString()} />
          </p>
          <p className="text-sm text-gray-500 mt-1">
            {overallStats.totalSessions} sessions
          </p>
        </div>
      </div>

      {/* Project List Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <h2 className="text-xl text-gray-200 flex items-center gap-2">
          <Folder className="w-5 h-5 text-blue-400" />
          Projects
          <span className="text-gray-500 text-sm ml-2">
            ({filteredProjects.length})
          </span>
        </h2>

        <div className="flex items-center gap-4">
          {/* Search */}
          <div className="relative">
            <Search className="w-4 h-4 text-gray-500 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder="Search projects..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9 pr-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-300 text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          {/* Sort Buttons */}
          <div className="flex items-center gap-2">
            <span className="text-gray-500 text-sm">Sort:</span>
            {(["activity", "cost", "tokens", "messages"] as const).map(
              (field) => (
                <button
                  key={field}
                  onClick={() => toggleSort(field)}
                  className={`px-2 py-1 rounded text-xs ${
                    sortBy === field
                      ? "bg-blue-600 text-white"
                      : "bg-gray-700 text-gray-400 hover:bg-gray-600"
                  }`}
                >
                  {field.charAt(0).toUpperCase() + field.slice(1)}
                  {sortBy === field && (sortOrder === "desc" ? " ↓" : " ↑")}
                </button>
              )
            )}
          </div>

        </div>
      </div>

      {/* Project List */}
      <div className="space-y-0">
        {filteredProjects.map((project) => (
          <ProjectRow key={project.projectPath} project={project} />
        ))}
      </div>

      {filteredProjects.length === 0 && searchQuery && (
        <div className="text-center py-8 text-gray-500">
          No projects match "{searchQuery}"
        </div>
      )}
    </div>
  );
}
