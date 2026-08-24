import React, { useState, useEffect } from 'react';

interface DrillInfo {
  id: string;
  name: string;
  path: string;
}

interface TrackInfo {
  id: string;
  name: string;
  stack: string;
  tier: string;
  description: string;
  drills: DrillInfo[];
}

interface ProgressData {
  total_xp: number;
  completed_drills: Array<{
    drill_path: string;
    track_id: string;
    score: number;
    completed_at: string;
  }>;
  unlocked_achievements: Array<{
    id: string;
    unlocked_at: string;
  }>;
  streak_days: number;
  consecutive_perfect_flakiness: number;
  perfect_locator_count: number;
}

const BADGES = [
  { id: 'first_blood', icon: '??', name: 'First Blood', desc: 'Complete your first drill' },
  { id: 'flakiness_slayer', icon: '??', name: 'Flakiness Slayer', desc: '3x 100/100 Flakiness under chaos' },
  { id: 'chaos_survivor', icon: '???', name: 'Chaos Survivor', desc: 'Pass 5/5 k6 load test iterations under chaos' },
  { id: 'tool_polyglot', icon: '??', name: 'Tool Polyglot', desc: 'Complete drills across 4 different tech stacks' },
  { id: 'the_architect', icon: '???', name: 'The Architect', desc: 'Master all Cross-Tool Decision drills' },
  { id: 'perfect_locator', icon: '??', name: 'Perfect Locator', desc: '5x 100/100 semantic locator scores' },
  { id: 'speed_demon', icon: '??', name: 'Speed Demon', desc: 'Beat execution speed baseline by 40%+' },
  { id: 'sdet_master', icon: '??', name: 'SDET Master', desc: 'Complete all 48 drills across all tracks' },
];

export const MissionControlPage: React.FC = () => {
  const [progress, setProgress] = useState<ProgressData | null>(null);
  const [curriculum, setCurriculum] = useState<TrackInfo[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [copiedPath, setCopiedPath] = useState<string | null>(null);

  // Chaos Console State
  const [chaosEndpoint, setChaosEndpoint] = useState<string>('/products');
  const [chaosHeader, setChaosHeader] = useState<string>('delay=500ms');
  const [chaosResponse, setChaosResponse] = useState<string | null>(null);
  const [chaosLoading, setChaosLoading] = useState<boolean>(false);
  const [chaosLatency, setChaosLatency] = useState<number | null>(null);

  useEffect(() => {
    // Fetch progress from backend
    fetch('http://localhost:8081/api/progress')
      .then((res) => res.json())
      .then((data) => setProgress(data))
      .catch(() => {
        setProgress({
          total_xp: 0,
          completed_drills: [],
          unlocked_achievements: [],
          streak_days: 0,
          consecutive_perfect_flakiness: 0,
          perfect_locator_count: 0,
        });
      });

    // Fetch curriculum tracks
    fetch('http://localhost:8081/api/curriculum')
      .then((res) => res.json())
      .then((data) => setCurriculum(data.tracks || []))
      .catch(() => {});
  }, []);

  const totalDrills = curriculum.reduce((acc, t) => acc + t.drills.length, 0) || 48;
  const completedCount = progress?.completed_drills?.length || 0;
  const xp = progress?.total_xp || 0;

  // Level thresholds
  const getRank = (xpVal: number) => {
    if (xpVal >= 20000) return { name: 'SDET Master', icon: '??', next: 20000, currentMin: 20000 };
    if (xpVal >= 10000) return { name: 'QA Architect', icon: '???', next: 20000, currentMin: 10000 };
    if (xpVal >= 6000) return { name: 'Lead QA', icon: '??', next: 10000, currentMin: 6000 };
    if (xpVal >= 3000) return { name: 'Senior QA', icon: '??', next: 6000, currentMin: 3000 };
    if (xpVal >= 1500) return { name: 'Mid QA', icon: '?', next: 3000, currentMin: 1500 };
    if (xpVal >= 500) return { name: 'Junior QA', icon: '??', next: 1500, currentMin: 500 };
    return { name: 'Trainee', icon: '??', next: 500, currentMin: 0 };
  };

  const rank = getRank(xp);
  const xpRange = rank.next - rank.currentMin;
  const xpProgress = rank.next === rank.currentMin ? 100 : Math.min(100, Math.max(0, ((xp - rank.currentMin) / xpRange) * 100));

  const handleCopy = (trackId: string, path: string) => {
    navigator.clipboard.writeText(`cherenkov-lings watch --track=${trackId}`);
    setCopiedPath(path);
    setTimeout(() => setCopiedPath(null), 2000);
  };

  const handleTestChaos = async () => {
    setChaosLoading(true);
    setChaosResponse(null);
    setChaosLatency(null);
    const start = Date.now();
    try {
      const res = await fetch(`http://localhost:8081${chaosEndpoint}`, {
        headers: chaosHeader ? { 'X-Chaos': chaosHeader } : {},
      });
      const data = await res.json();
      const elapsed = Date.now() - start;
      setChaosLatency(elapsed);
      setChaosResponse(JSON.stringify(data, null, 2));
    } catch (err: any) {
      const elapsed = Date.now() - start;
      setChaosLatency(elapsed);
      setChaosResponse(`Error: ${err.message}`);
    } finally {
      setChaosLoading(false);
    }
  };

  const filteredTracks = curriculum.filter((t) => {
    if (selectedCategory === 'all') return true;
    if (selectedCategory === 'ui') return t.id === 'playwright-ts';
    if (selectedCategory === 'api') return t.id === 'restassured-java';
    if (selectedCategory === 'mobile') return t.id === 'maestro-mobile';
    if (selectedCategory === 'perf') return t.id === 'k6-js' || t.id === 'jmeter';
    if (selectedCategory === 'foundations') return t.id === 'foundations';
    if (selectedCategory === 'security') return t.id === 'devsecops-python' || t.id === 'genai-qa';
    if (selectedCategory === 'architecture') return t.id === 'tool-decisions';
    return true;
  });

  return (
    <div className="page-container" style={{ maxWidth: '1200px', margin: '0 auto', padding: '24px' }}>
      {/* Header Banner */}
      <div style={{ background: 'linear-gradient(135deg, #0d1b2a 0%, #1b263b 100%)', borderRadius: '16px', padding: '32px', border: '1px solid #415a77', marginBottom: '32px', color: '#e0e1dd' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '20px' }}>
          <div>
            <span style={{ background: '#0077b6', color: '#fff', padding: '4px 12px', borderRadius: '20px', fontSize: '12px', fontWeight: 'bold', textTransform: 'uppercase' }}>Mission Control & Gamified Learning</span>
            <h1 style={{ fontSize: '32px', margin: '8px 0 4px 0', color: '#fff' }}>SDET Proving Ground</h1>
            <p style={{ margin: 0, color: '#778da9', fontSize: '15px' }}>Master test automation from scratch to architecture across UI, API, Mobile, Performance, Security & LLM testing.</p>
          </div>
          <div style={{ display: 'flex', gap: '16px', alignItems: 'center', background: '#0a1128', padding: '16px 24px', borderRadius: '12px', border: '1px solid #0077b6' }}>
            <div style={{ fontSize: '36px' }}>{rank.icon}</div>
            <div>
              <div style={{ fontSize: '12px', color: '#778da9', textTransform: 'uppercase' }}>Current Rank</div>
              <div style={{ fontSize: '20px', fontWeight: 'bold', color: '#90e0ef' }}>{rank.name}</div>
              <div style={{ fontSize: '13px', color: '#00b4d8' }}>{xp.toLocaleString()} XP Total</div>
            </div>
          </div>
        </div>

        {/* Level Progress Bar */}
        <div style={{ marginTop: '24px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '13px', marginBottom: '6px', color: '#e0e1dd' }}>
            <span>Rank Progression</span>
            <span>{xp} / {rank.next} XP ({xpProgress.toFixed(1)}%)</span>
          </div>
          <div style={{ height: '12px', background: '#0a1128', borderRadius: '6px', overflow: 'hidden', border: '1px solid #415a77' }}>
            <div style={{ height: '100%', width: `${xpProgress}%`, background: 'linear-gradient(90deg, #0077b6 0%, #00b4d8 100%)', borderRadius: '6px', transition: 'width 0.5s ease' }}></div>
          </div>
        </div>
      </div>

      {/* Stats Cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '16px', marginBottom: '32px' }}>
        <div style={{ background: '#1b263b', padding: '20px', borderRadius: '12px', border: '1px solid #415a77' }}>
          <div style={{ fontSize: '13px', color: '#778da9' }}>?? Active Streak</div>
          <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#ffb703', marginTop: '4px' }}>{progress?.streak_days || 0} Days</div>
          <div style={{ fontSize: '12px', color: '#778da9', marginTop: '4px' }}>Daily practice builds mastery</div>
        </div>
        <div style={{ background: '#1b263b', padding: '20px', borderRadius: '12px', border: '1px solid #415a77' }}>
          <div style={{ fontSize: '13px', color: '#778da9' }}>?? Drills Completed</div>
          <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#52b788', marginTop: '4px' }}>{completedCount} / {totalDrills}</div>
          <div style={{ fontSize: '12px', color: '#778da9', marginTop: '4px' }}>Across 9 specialized tracks</div>
        </div>
        <div style={{ background: '#1b263b', padding: '20px', borderRadius: '12px', border: '1px solid #415a77' }}>
          <div style={{ fontSize: '13px', color: '#778da9' }}>?? Badges Unlocked</div>
          <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#90e0ef', marginTop: '4px' }}>{progress?.unlocked_achievements?.length || 0} / {BADGES.length}</div>
          <div style={{ fontSize: '12px', color: '#778da9', marginTop: '4px' }}>Specialist achievements</div>
        </div>
        <div style={{ background: '#1b263b', padding: '20px', borderRadius: '12px', border: '1px solid #415a77' }}>
          <div style={{ fontSize: '13px', color: '#778da9' }}>? Chaos Resistance</div>
          <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#f72585', marginTop: '4px' }}>{progress?.consecutive_perfect_flakiness || 0}x</div>
          <div style={{ fontSize: '12px', color: '#778da9', marginTop: '4px' }}>Consecutive 100/100 runs</div>
        </div>
      </div>

      {/* Badges Section */}
      <div style={{ background: '#1b263b', padding: '24px', borderRadius: '16px', border: '1px solid #415a77', marginBottom: '32px' }}>
        <h2 style={{ fontSize: '20px', color: '#fff', marginTop: 0, marginBottom: '16px' }}>??? Achievement Showcase</h2>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '12px' }}>
          {BADGES.map((b) => {
            const isUnlocked = progress?.unlocked_achievements?.some((ua) => ua.id === b.id);
            return (
              <div
                key={b.id}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                  padding: '12px 16px',
                  borderRadius: '10px',
                  background: isUnlocked ? 'rgba(0, 180, 216, 0.15)' : 'rgba(255, 255, 255, 0.03)',
                  border: isUnlocked ? '1px solid #00b4d8' : '1px solid rgba(255, 255, 255, 0.08)',
                  opacity: isUnlocked ? 1 : 0.6,
                }}
              >
                <div style={{ fontSize: '28px' }}>{b.icon}</div>
                <div>
                  <div style={{ fontWeight: 'bold', color: isUnlocked ? '#90e0ef' : '#778da9', fontSize: '14px' }}>{b.name}</div>
                  <div style={{ fontSize: '12px', color: '#778da9' }}>{b.desc}</div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Live Chaos Simulator Sandbox */}
      <div style={{ background: '#0d1b2a', padding: '24px', borderRadius: '16px', border: '1px solid #0077b6', marginBottom: '32px', color: '#e0e1dd' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
          <div>
            <h2 style={{ fontSize: '20px', color: '#fff', margin: '0 0 4px 0' }}>?? Live Micro-Crucible Chaos Console</h2>
            <p style={{ margin: 0, color: '#778da9', fontSize: '13px' }}>Simulate real-world network and application pathologies before writing automation assertions.</p>
          </div>
          <span style={{ background: '#52b788', color: '#0d1b2a', padding: '4px 10px', borderRadius: '12px', fontSize: '12px', fontWeight: 'bold' }}>PORT 8081 CONNECTED</span>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px', marginBottom: '16px' }}>
          <div>
            <label style={{ display: 'block', fontSize: '13px', color: '#90e0ef', marginBottom: '6px' }}>Target API Endpoint:</label>
            <select
              value={chaosEndpoint}
              onChange={(e) => setChaosEndpoint(e.target.value)}
              style={{ width: '100%', padding: '10px', background: '#1b263b', border: '1px solid #415a77', borderRadius: '8px', color: '#fff' }}
            >
              <option value="/products?page=1&per_page=5">GET /products (Paginated Catalog)</option>
              <option value="/checkout">GET /checkout (State & Cart)</option>
              <option value="/transfer">GET /transfer (Ledger & Accounts)</option>
              <option value="/search?q=Playwright">GET /search (Autocomplete Debounce)</option>
              <option value="/api/rag?query=test">GET /api/rag (Deterministic RAG Grounding)</option>
              <option value="/api/llm?query=test">GET /api/llm (Non-Deterministic LLM Output)</option>
            </select>
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '13px', color: '#90e0ef', marginBottom: '6px' }}>X-Chaos Fault Injection Header:</label>
            <select
              value={chaosHeader}
              onChange={(e) => setChaosHeader(e.target.value)}
              style={{ width: '100%', padding: '10px', background: '#1b263b', border: '1px solid #415a77', borderRadius: '8px', color: '#fff' }}
            >
              <option value="delay=500ms">delay=500ms (Artificial Latency)</option>
              <option value="delay=1200ms;jitter=300ms">delay=1200ms;jitter=300ms (Network Jitter Spike)</option>
              <option value="kafka_lag=1500">kafka_lag=1500 (Eventual Consistency Delay)</option>
              <option value="token_expire=immediate">token_expire=immediate (JWT Session Invalidation)</option>
              <option value="stale_dom=true">stale_dom=true (DOM Replacement Race)</option>
              <option value="">(No Chaos — Clean Run)</option>
            </select>
          </div>
        </div>

        <div style={{ display: 'flex', gap: '12px', alignItems: 'center', marginBottom: '16px' }}>
          <button
            onClick={handleTestChaos}
            disabled={chaosLoading}
            style={{
              background: '#0077b6',
              color: '#fff',
              border: 'none',
              padding: '10px 24px',
              borderRadius: '8px',
              fontWeight: 'bold',
              cursor: 'pointer',
              opacity: chaosLoading ? 0.7 : 1,
            }}
          >
            {chaosLoading ? 'Injecting Chaos...' : '?? Fire Chaos Request'}
          </button>
          {chaosLatency !== null && (
            <span style={{ fontSize: '13px', color: chaosLatency > 500 ? '#ffb703' : '#52b788', fontWeight: 'bold' }}>
              ?? Latency: {chaosLatency}ms
            </span>
          )}
        </div>

        {chaosResponse && (
          <div style={{ background: '#050a14', padding: '16px', borderRadius: '8px', border: '1px solid #1b263b', maxHeight: '200px', overflowY: 'auto' }}>
            <pre style={{ margin: 0, fontSize: '12px', color: '#90e0ef' }}>{chaosResponse}</pre>
          </div>
        )}
      </div>

      {/* Curriculum Tracks Explorer */}
      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '16px', marginBottom: '20px' }}>
          <h2 style={{ fontSize: '24px', color: '#fff', margin: 0 }}>?? 48-Drill Curriculum Explorer</h2>

          {/* Category Filter Pills */}
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
            {[
              { id: 'all', label: 'All Tracks' },
              { id: 'foundations', label: 'Foundations' },
              { id: 'ui', label: 'Web UI' },
              { id: 'api', label: 'API / Backend' },
              { id: 'mobile', label: 'Mobile' },
              { id: 'perf', label: 'Performance' },
              { id: 'security', label: 'DevSecOps & AI' },
              { id: 'architecture', label: 'Tool Decisions' },
            ].map((cat) => (
              <button
                key={cat.id}
                onClick={() => setSelectedCategory(cat.id)}
                style={{
                  background: selectedCategory === cat.id ? '#0077b6' : '#1b263b',
                  color: selectedCategory === cat.id ? '#fff' : '#778da9',
                  border: '1px solid #415a77',
                  padding: '6px 14px',
                  borderRadius: '20px',
                  fontSize: '13px',
                  cursor: 'pointer',
                  fontWeight: selectedCategory === cat.id ? 'bold' : 'normal',
                }}
              >
                {cat.label}
              </button>
            ))}
          </div>
        </div>

        {/* Tracks List */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {filteredTracks.map((track) => (
            <div key={track.id} style={{ background: '#1b263b', borderRadius: '16px', padding: '24px', border: '1px solid #415a77' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '12px', marginBottom: '16px' }}>
                <div>
                  <div style={{ display: 'flex', gap: '8px', alignItems: 'center', marginBottom: '6px' }}>
                    <h3 style={{ fontSize: '18px', margin: 0, color: '#fff' }}>{track.name}</h3>
                    <span style={{ background: '#0077b6', color: '#fff', fontSize: '11px', padding: '2px 8px', borderRadius: '12px', fontWeight: 'bold' }}>{track.stack}</span>
                    <span style={{ background: '#415a77', color: '#e0e1dd', fontSize: '11px', padding: '2px 8px', borderRadius: '12px' }}>{track.tier}</span>
                  </div>
                  <p style={{ margin: 0, color: '#778da9', fontSize: '14px' }}>{track.description}</p>
                </div>
                <button
                  onClick={() => handleCopy(track.id, track.id)}
                  style={{
                    background: '#0d1b2a',
                    border: '1px solid #00b4d8',
                    color: '#90e0ef',
                    padding: '8px 16px',
                    borderRadius: '8px',
                    fontSize: '13px',
                    fontWeight: 'bold',
                    cursor: 'pointer',
                  }}
                >
                  {copiedPath === track.id ? '? Command Copied!' : `? Watch Track`}
                </button>
              </div>

              {/* Drills Grid */}
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '12px' }}>
                {track.drills.map((drill, idx) => (
                  <div
                    key={drill.id}
                    style={{
                      background: '#0d1b2a',
                      padding: '14px 16px',
                      borderRadius: '10px',
                      border: '1px solid #415a77',
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                    }}
                  >
                    <div>
                      <div style={{ fontSize: '11px', color: '#00b4d8', fontWeight: 'bold', textTransform: 'uppercase' }}>Drill #{idx + 1}</div>
                      <div style={{ fontSize: '14px', fontWeight: 'bold', color: '#e0e1dd', marginTop: '2px' }}>{drill.name}</div>
                      <div style={{ fontSize: '11px', color: '#778da9', marginTop: '4px', fontFamily: 'monospace' }}>{drill.path}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
