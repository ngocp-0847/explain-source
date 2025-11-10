'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { CheckCircle, XCircle, Edit3, Save, Users } from 'lucide-react'
import { useAuthStore } from '@/stores/authStore'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

interface PlanApproval {
  id: string
  ticket_id: string
  user_id: string
  approved_at: string
  status: 'approved' | 'rejected'
}

interface PlanEditorProps {
  ticketId: string
  planContent?: string
  requiredApprovals: number
  onPlanUpdate?: (content: string) => void
}

export function PlanEditor({ ticketId, planContent = '', requiredApprovals, onPlanUpdate }: PlanEditorProps) {
  const [isEditing, setIsEditing] = useState(false)
  const [editContent, setEditContent] = useState(planContent)
  const [approvals, setApprovals] = useState<PlanApproval[]>([])
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const { user, token } = useAuthStore()

  useEffect(() => {
    setEditContent(planContent)
  }, [planContent])

  // Load approvals
  useEffect(() => {
    const loadApprovals = async () => {
      try {
        const response = await fetch(`http://localhost:9000/api/tickets/${ticketId}/plan/approvals`, {
          headers: {
            'Authorization': `Bearer ${token}`,
          },
        })
        if (response.ok) {
          const data = await response.json()
          setApprovals(data)
        }
      } catch (error) {
        console.error('Failed to load approvals:', error)
      }
    }

    if (token) {
      loadApprovals()
    }
  }, [ticketId, token])

  const handleSave = async () => {
    if (!token) {
      alert('Vui lòng đăng nhập để edit plan')
      return
    }

    setSaving(true)
    try {
      const response = await fetch(`http://localhost:9000/api/tickets/${ticketId}/plan`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify({ content: editContent }),
      })

      if (response.ok) {
        setIsEditing(false)
        onPlanUpdate?.(editContent)
        alert('Plan đã được cập nhật!')
      } else {
        const error = await response.json()
        alert(`Lỗi: ${error.error}`)
      }
    } catch (error) {
      console.error('Failed to update plan:', error)
      alert('Lỗi khi cập nhật plan')
    } finally {
      setSaving(false)
    }
  }

  const handleApprove = async (status: 'approved' | 'rejected') => {
    if (!token) {
      alert('Vui lòng đăng nhập để approve plan')
      return
    }

    setLoading(true)
    try {
      const response = await fetch(`http://localhost:9000/api/tickets/${ticketId}/plan/approve`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify({ status }),
      })

      if (response.ok) {
        // Reload approvals
        const approvalsResponse = await fetch(`http://localhost:9000/api/tickets/${ticketId}/plan/approvals`, {
          headers: {
            'Authorization': `Bearer ${token}`,
          },
        })
        if (approvalsResponse.ok) {
          const data = await approvalsResponse.json()
          setApprovals(data)
        }
        alert(`Plan đã được ${status === 'approved' ? 'approve' : 'reject'}!`)
      } else {
        const error = await response.json()
        alert(`Lỗi: ${error.error}`)
      }
    } catch (error) {
      console.error('Failed to approve plan:', error)
      alert('Lỗi khi approve plan')
    } finally {
      setLoading(false)
    }
  }

  const approvedCount = approvals.filter(a => a.status === 'approved').length
  const userApproval = approvals.find(a => a.user_id === user?.id)
  const canImplement = approvedCount >= requiredApprovals

  return (
    <Card className="border-blue-200 bg-blue-50/30">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg flex items-center gap-2">
            📝 Implementation Plan
            {canImplement && (
              <Badge className="bg-green-600">
                ✓ Đã đủ approvals - Sẵn sàng implement
              </Badge>
            )}
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="flex items-center gap-1">
              <Users className="w-3 h-3" />
              {approvedCount}/{requiredApprovals} approvals
            </Badge>
            {!isEditing && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => setIsEditing(true)}
                disabled={!user}
              >
                <Edit3 className="w-4 h-4 mr-1" />
                Edit
              </Button>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {isEditing ? (
          <div className="space-y-2">
            <Textarea
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              placeholder="Viết plan implementation (markdown supported)..."
              rows={15}
              className="font-mono text-sm"
            />
            <div className="flex gap-2">
              <Button onClick={handleSave} disabled={saving}>
                <Save className="w-4 h-4 mr-1" />
                {saving ? 'Đang lưu...' : 'Lưu Plan'}
              </Button>
              <Button
                variant="outline"
                onClick={() => {
                  setEditContent(planContent)
                  setIsEditing(false)
                }}
              >
                Hủy
              </Button>
            </div>
          </div>
        ) : (
          <div>
            {planContent ? (
              <div className="prose prose-sm max-w-none bg-white p-4 rounded-lg border">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                  {planContent}
                </ReactMarkdown>
              </div>
            ) : (
              <div className="text-center text-gray-500 py-8 bg-white rounded-lg border border-dashed">
                Chưa có plan. Click &quot;Edit&quot; để tạo plan.
              </div>
            )}
          </div>
        )}

        {/* Approval Section */}
        {planContent && (
          <div className="border-t pt-4 space-y-3">
            <h4 className="font-semibold text-sm">Approval Status</h4>
            
            {!userApproval && user ? (
              <div className="flex gap-2">
                <Button
                  onClick={() => handleApprove('approved')}
                  disabled={loading}
                  className="bg-green-600 hover:bg-green-700"
                >
                  <CheckCircle className="w-4 h-4 mr-1" />
                  Approve
                </Button>
                <Button
                  onClick={() => handleApprove('rejected')}
                  disabled={loading}
                  variant="destructive"
                >
                  <XCircle className="w-4 h-4 mr-1" />
                  Reject
                </Button>
              </div>
            ) : userApproval ? (
              <Badge className={userApproval.status === 'approved' ? 'bg-green-600' : 'bg-red-600'}>
                Bạn đã {userApproval.status === 'approved' ? 'approve' : 'reject'} plan này
              </Badge>
            ) : (
              <p className="text-sm text-gray-600">Đăng nhập để approve plan</p>
            )}

            {/* Show all approvals */}
            {approvals.length > 0 && (
              <div className="space-y-1">
                <p className="text-xs text-gray-600">Người đã approve:</p>
                <div className="flex flex-wrap gap-2">
                  {approvals.filter(a => a.status === 'approved').map(approval => (
                    <Badge key={approval.id} variant="outline" className="text-green-700 border-green-300">
                      ✓ User {approval.user_id.substring(0, 8)}...
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  )
}

