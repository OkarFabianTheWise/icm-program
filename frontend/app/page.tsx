"use client"

import type React from "react"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Badge } from "@/components/ui/badge"
import { 
  Plus, 
  TrendingUp, 
  Wallet, 
  Gift,
  Users,
  Clock,
  DollarSign,
  ArrowUpRight,
  Trophy,
  Target
} from "lucide-react"

interface Bucket {
  id: string
  name: string
  creator: string
  status: "raising" | "trading" | "closed"
  totalContributions: number
  contributionDeadline: Date
  tradingDeadline: Date
  tokenMints: string[]
  creatorFeePercent: number
  contributors: number
}

export default function ICMProgramInterface() {
  const [activeTab, setActiveTab] = useState("buckets")
  const [isConnected, setIsConnected] = useState(false)
  const [walletAddress] = useState("")
  const [buckets] = useState<Bucket[]>([
    {
      id: "1",
      name: "DeFi Growth Fund",
      creator: "Ax4...K2P",
      status: "raising",
      totalContributions: 15420.50,
      contributionDeadline: new Date(Date.now() + 5 * 24 * 60 * 60 * 1000),
      tradingDeadline: new Date(Date.now() + 35 * 24 * 60 * 60 * 1000),
      tokenMints: ["SOL", "USDC", "RAY"],
      creatorFeePercent: 5,
      contributors: 23
    },
    {
      id: "2", 
      name: "Meme Coin Alpha",
      creator: "Bz8...N9X",
      status: "trading",
      totalContributions: 8950.25,
      contributionDeadline: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000),
      tradingDeadline: new Date(Date.now() + 28 * 24 * 60 * 60 * 1000),
      tokenMints: ["BONK", "WIF", "POPCAT"],
      creatorFeePercent: 3,
      contributors: 15
    },
    {
      id: "3",
      name: "Blue Chip Basket",
      creator: "Cx2...M4L",
      status: "closed",
      totalContributions: 32100.75,
      contributionDeadline: new Date(Date.now() - 35 * 24 * 60 * 60 * 1000),
      tradingDeadline: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000),
      tokenMints: ["SOL", "ETH", "BTC"],
      creatorFeePercent: 2,
      contributors: 67
    }
  ])

  const [createBucketForm, setCreateBucketForm] = useState({
    name: "",
    contributionDays: "7",
    tradingDays: "30",
    creatorFee: "5",
    tokenMints: [""]
  })

  const [contributeForm, setContributeForm] = useState({
    bucketId: "",
    tokenMint: "",
    amount: ""
  })

  const connectWallet = () => {
    setIsConnected(true)
    // Here you'd integrate with Solana wallet adapter
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case "raising": return "bg-green-500/20 text-green-400 border-green-500/30"
      case "trading": return "bg-blue-500/20 text-blue-400 border-blue-500/30"
      case "closed": return "bg-gray-500/20 text-gray-400 border-gray-500/30"
      default: return "bg-gray-500/20 text-gray-400 border-gray-500/30"
    }
  }

  const formatTimeRemaining = (date: Date) => {
    const now = new Date()
    const diff = date.getTime() - now.getTime()
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))
    if (days < 0) return "Expired"
    if (days === 0) return "< 1 day"
    return `${days} days`
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#001532] via-[#001a3d] to-[#023585] p-4 font-inter">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">ICM Protocol</h1>
            <p className="text-blue-200/70">Investment Club Management on Solana</p>
          </div>
          <Button
            onClick={connectWallet}
            className={`${
              isConnected
                ? "bg-green-500/20 text-green-400 border-green-500/30"
                : "bg-blue-500 hover:bg-blue-600"
            } border rounded-xl px-6 py-2 transition-all duration-200`}
          >
            <Wallet className="w-4 h-4 mr-2" />
            {isConnected ? `${walletAddress.slice(0, 4)}...${walletAddress.slice(-4)}` : "Connect Wallet"}
          </Button>
        </div>

        {/* Main Content */}
        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="bg-black/20 backdrop-blur-xl border border-white/10 rounded-xl p-1 mb-8">
            <TabsTrigger value="buckets" className="data-[state=active]:bg-blue-500/20 data-[state=active]:text-blue-400">
              <Target className="w-4 h-4 mr-2" />
              View Buckets
            </TabsTrigger>
            <TabsTrigger value="create" className="data-[state=active]:bg-blue-500/20 data-[state=active]:text-blue-400">
              <Plus className="w-4 h-4 mr-2" />
              Create Bucket
            </TabsTrigger>
            <TabsTrigger value="contribute" className="data-[state=active]:bg-blue-500/20 data-[state=active]:text-blue-400">
              <DollarSign className="w-4 h-4 mr-2" />
              Contribute
            </TabsTrigger>
            <TabsTrigger value="rewards" className="data-[state=active]:bg-blue-500/20 data-[state=active]:text-blue-400">
              <Trophy className="w-4 h-4 mr-2" />
              Claim Rewards
            </TabsTrigger>
          </TabsList>

          {/* View Buckets Tab */}
          <TabsContent value="buckets" className="space-y-6">
            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
              {buckets.map((bucket) => (
                <Card key={bucket.id} className="backdrop-blur-xl bg-black/20 border border-white/10 rounded-2xl hover:bg-black/25 transition-all duration-300">
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div>
                        <CardTitle className="text-white text-lg">{bucket.name}</CardTitle>
                        <CardDescription className="text-blue-200/70">
                          by {bucket.creator}
                        </CardDescription>
                      </div>
                      <Badge className={`${getStatusColor(bucket.status)} border`}>
                        {bucket.status}
                      </Badge>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <p className="text-blue-200/70">Total Raised</p>
                        <p className="text-white font-semibold">${bucket.totalContributions.toLocaleString()}</p>
                      </div>
                      <div>
                        <p className="text-blue-200/70">Contributors</p>
                        <p className="text-white font-semibold">{bucket.contributors}</p>
                      </div>
                    </div>
                    
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span className="text-blue-200/70">Tokens:</span>
                        <span className="text-white">{bucket.tokenMints.join(", ")}</span>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span className="text-blue-200/70">Creator Fee:</span>
                        <span className="text-white">{bucket.creatorFeePercent}%</span>
                      </div>
                    </div>

                    <div className="flex justify-between text-sm">
                      <span className="text-blue-200/70 flex items-center gap-1">
                        <Clock className="w-3 h-3" />
                        {bucket.status === "raising" ? "Contribution ends" : 
                         bucket.status === "trading" ? "Trading ends" : "Ended"}
                      </span>
                      <span className="text-white">
                        {bucket.status === "raising" 
                          ? formatTimeRemaining(bucket.contributionDeadline)
                          : bucket.status === "trading"
                          ? formatTimeRemaining(bucket.tradingDeadline)
                          : "Closed"
                        }
                      </span>
                    </div>

                    {bucket.status === "trading" && (
                      <Button className="w-full bg-blue-500/20 text-blue-400 border border-blue-500/30 hover:bg-blue-500/30 rounded-xl">
                        <TrendingUp className="w-4 h-4 mr-2" />
                        View Trading
                      </Button>
                    )}
                  </CardContent>
                </Card>
              ))}
            </div>
          </TabsContent>

          {/* Create Bucket Tab */}
          <TabsContent value="create">
            <Card className="backdrop-blur-xl bg-black/20 border border-white/10 rounded-2xl p-8 max-w-2xl mx-auto">
              <CardHeader className="px-0 pt-0">
                <CardTitle className="text-white text-2xl">Create New Bucket</CardTitle>
                <CardDescription className="text-blue-200/70">
                  Set up a new investment club bucket for collaborative trading
                </CardDescription>
              </CardHeader>
              <CardContent className="px-0 space-y-6">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-blue-200">Bucket Name</label>
                  <Input
                    placeholder="Enter bucket name..."
                    value={createBucketForm.name}
                    onChange={(e) => setCreateBucketForm({...createBucketForm, name: e.target.value})}
                    className="bg-black/30 border-white/20 text-white placeholder:text-white/40 focus:border-blue-400 rounded-xl h-12"
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <label className="text-sm font-medium text-blue-200">Contribution Window</label>
                    <Select value={createBucketForm.contributionDays} onValueChange={(value) => setCreateBucketForm({...createBucketForm, contributionDays: value})}>
                      <SelectTrigger className="bg-black/30 border-white/20 text-white rounded-xl h-12">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent className="bg-black/90 backdrop-blur-xl border-white/20">
                        <SelectItem value="3">3 days</SelectItem>
                        <SelectItem value="7">7 days</SelectItem>
                        <SelectItem value="14">14 days</SelectItem>
                        <SelectItem value="30">30 days</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium text-blue-200">Trading Window</label>
                    <Select value={createBucketForm.tradingDays} onValueChange={(value) => setCreateBucketForm({...createBucketForm, tradingDays: value})}>
                      <SelectTrigger className="bg-black/30 border-white/20 text-white rounded-xl h-12">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent className="bg-black/90 backdrop-blur-xl border-white/20">
                        <SelectItem value="7">7 days</SelectItem>
                        <SelectItem value="30">30 days</SelectItem>
                        <SelectItem value="60">60 days</SelectItem>
                        <SelectItem value="90">90 days</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-blue-200">Creator Fee (%)</label>
                  <Input
                    type="number"
                    placeholder="5"
                    min="0"
                    max="20"
                    value={createBucketForm.creatorFee}
                    onChange={(e) => setCreateBucketForm({...createBucketForm, creatorFee: e.target.value})}
                    className="bg-black/30 border-white/20 text-white placeholder:text-white/40 focus:border-blue-400 rounded-xl h-12"
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-blue-200">Token Mints (comma separated)</label>
                  <Input
                    placeholder="SOL, USDC, RAY..."
                    className="bg-black/30 border-white/20 text-white placeholder:text-white/40 focus:border-blue-400 rounded-xl h-12"
                  />
                </div>

                <Button className="w-full bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white font-semibold py-3 rounded-xl h-12">
                  <Plus className="w-4 h-4 mr-2" />
                  Create Bucket
                </Button>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Contribute Tab */}
          <TabsContent value="contribute">
            <Card className="backdrop-blur-xl bg-black/20 border border-white/10 rounded-2xl p-8 max-w-2xl mx-auto">
              <CardHeader className="px-0 pt-0">
                <CardTitle className="text-white text-2xl">Contribute to Bucket</CardTitle>
                <CardDescription className="text-blue-200/70">
                  Add funds to an active bucket during its contribution window
                </CardDescription>
              </CardHeader>
              <CardContent className="px-0 space-y-6">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-blue-200">Select Bucket</label>
                  <Select value={contributeForm.bucketId} onValueChange={(value) => setContributeForm({...contributeForm, bucketId: value})}>
                    <SelectTrigger className="bg-black/30 border-white/20 text-white rounded-xl h-12">
                      <SelectValue placeholder="Choose a bucket..." />
                    </SelectTrigger>
                    <SelectContent className="bg-black/90 backdrop-blur-xl border-white/20">
                      {buckets.filter(b => b.status === "raising").map(bucket => (
                        <SelectItem key={bucket.id} value={bucket.id}>
                          {bucket.name} - ${bucket.totalContributions.toLocaleString()}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-blue-200">Token</label>
                  <Select value={contributeForm.tokenMint} onValueChange={(value) => setContributeForm({...contributeForm, tokenMint: value})}>
                    <SelectTrigger className="bg-black/30 border-white/20 text-white rounded-xl h-12">
                      <SelectValue placeholder="Select token..." />
                    </SelectTrigger>
                    <SelectContent className="bg-black/90 backdrop-blur-xl border-white/20">
                      <SelectItem value="SOL">SOL</SelectItem>
                      <SelectItem value="USDC">USDC</SelectItem>
                      <SelectItem value="USDT">USDT</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-blue-200">Amount</label>
                  <Input
                    type="number"
                    placeholder="0.00"
                    step="0.000001"
                    min="0"
                    value={contributeForm.amount}
                    onChange={(e) => setContributeForm({...contributeForm, amount: e.target.value})}
                    className="bg-black/30 border-white/20 text-white placeholder:text-white/40 focus:border-blue-400 rounded-xl h-12"
                  />
                </div>

                <Button className="w-full bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 text-white font-semibold py-3 rounded-xl h-12">
                  <DollarSign className="w-4 h-4 mr-2" />
                  Contribute
                </Button>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Claim Rewards Tab */}
          <TabsContent value="rewards">
            <Card className="backdrop-blur-xl bg-black/20 border border-white/10 rounded-2xl p-8 max-w-2xl mx-auto">
              <CardHeader className="px-0 pt-0">
                <CardTitle className="text-white text-2xl">Claim Rewards</CardTitle>
                <CardDescription className="text-blue-200/70">
                  Claim your rewards from closed buckets
                </CardDescription>
              </CardHeader>
              <CardContent className="px-0 space-y-6">
                <div className="space-y-4">
                  {buckets.filter(b => b.status === "closed").map(bucket => (
                    <div key={bucket.id} className="bg-black/30 rounded-xl p-4 border border-white/10">
                      <div className="flex items-center justify-between mb-3">
                        <h4 className="text-white font-semibold">{bucket.name}</h4>
                        <Badge className="bg-purple-500/20 text-purple-400 border-purple-500/30">
                          Rewards Available
                        </Badge>
                      </div>
                      <div className="grid grid-cols-3 gap-4 text-sm mb-4">
                        <div>
                          <p className="text-blue-200/70">Your Contribution</p>
                          <p className="text-white font-semibold">$1,250</p>
                        </div>
                        <div>
                          <p className="text-blue-200/70">Current Value</p>
                          <p className="text-green-400 font-semibold">$1,487</p>
                        </div>
                        <div>
                          <p className="text-blue-200/70">Profit</p>
                          <p className="text-green-400 font-semibold">+$237 (19%)</p>
                        </div>
                      </div>
                      <Button className="w-full bg-gradient-to-r from-purple-500 to-purple-600 hover:from-purple-600 hover:to-purple-700 text-white font-semibold py-2 rounded-xl">
                        <Gift className="w-4 h-4 mr-2" />
                        Claim Rewards
                      </Button>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}
